import hashlib
from dslib import Context, Message, Node
from typing import List

def is_newer(context1, value1, context2, value2):
    return context1 > context2 or (context1 == context2 and (value2 is None or (value1 is not None and value1 > value2)))

class RequestContext:
    def __init__(self, ctx, nodes, node_id, req_id, key, value, quorum, msg_type: str):
        self.nodes = nodes
        self.node_id = node_id
        self.key = key
        self.value = value
        self.id = req_id
        self.acks = 0
        self.quorum = quorum
        self.responses = []
        self.msg_type = msg_type
        self.req_msg_type = msg_type + "_REQ"
        self.resp_msg_type = msg_type + "_RESP"
        self.context = ctx.time()

        self.delivered = False
        self.synchronized = False
        self.final_context = -1
        self.final_value = ''
        
        self.replica_idxs = get_key_replicas(self.key, len(self.nodes))
        self.replicas = [self.nodes[x] for x in self.replica_idxs]
        self.next_replica_idx = self.replica_idxs[-1]

        self.ack_replicas = set()
        self.missing_replicas = set()

        self.extra_replicas = set()

        self.retry_time = 0.5

        print('{0} {1}->{2} with quorum={3}'.format(self.msg_type, self.node_id, self.replicas, self.quorum))
        for node in self.replicas:
            self.send_req(ctx, node)

    def ack(self, ctx: Context, msg: Message, sender):
        ctx.cancel_timer(self.timer_name(sender))
        if sender in self.ack_replicas:
            return

        if sender in self.replicas:
            self.ack_replicas.add(sender)
            if not self.synchronized and len(self.ack_replicas) == len(self.replicas):
                self.synchronized = True
                for node in self.extra_replicas:
                    repair = Message('REPAIR_DELETE', {
                        'id': self.id,
                        'key': self.key,
                        'context': self.context
                    })
                    ctx.send(repair, sender)
        else:
            self.extra_replicas.add(sender)

        if self.synchronized and sender not in self.replicas:
            repair = Message('REPAIR_DELETE', {
                'id': self.id,
                'key': self.key,
                'context': self.context
            })
            ctx.send(repair, sender)
            return

        if self.delivered:
            if msg.type == 'GET_ACK':
                if sender in self.replicas:
                    if is_newer(self.final_context, self.final_value, msg['context'], msg['value']) and self.final_value != msg['value']: 
                        repair = Message('REPAIR_UPDATE', {
                            'id': self.id,
                            'key': self.key,
                            'value': self.final_value,
                            'context': self.final_context
                        })
                        ctx.send(repair, sender)
                    
            return
            
        self.acks += 1
        self.responses.append((msg['context'], msg['value'], sender))
        
        print('ACK {2} {0}->{1}, id={3}, data[{4}, {5}]={6}, acks/quorum={7}/{8}'.format(
                sender, self.node_id, self.msg_type, self.id, msg['context'], msg['key'], msg['value'], self.acks, self.quorum))

        if self.acks >= self.quorum:
            self.final_value = None
            self.final_context = -1
            for context, value, node in self.responses:
                if is_newer(context, value, self.final_context,  self.final_value):
                    self.final_value = value
                    self.final_context = context

            if msg.type == 'GET_ACK':
                repair = Message('REPAIR_UPDATE', {
                    'id': self.id,
                    'key': self.key,
                    'value': self.final_value,
                    'context': self.final_context,
                })

                for context, value, node in self.responses:
                    if is_newer(self.final_context,  self.final_value, context, value) and self.final_value != value:
                        ctx.send(repair, node)
                for node in self.missing_replicas:
                    ctx.send(repair, node)

            resp = Message(self.resp_msg_type, {
                'key': self.key,
                'value': self.final_value 
            })
            print(self.resp_msg_type, self.key, self.final_value )
            print()
            ctx.send_local(resp)
            self.delivered = True

    def timer_name(self, node):
        return self.req_msg_type + '$' + node + '$' + self.id

    def send_req(self, ctx, node):
        req = {}
        req = Message(self.req_msg_type, {
            'id': self.id,
            'key': self.key,
            'value': self.value,
            'context': self.context,
            'quorum': self.quorum, # debug info
        })
        ctx.send(req, node)
        ctx.set_timer(self.timer_name(node), 1)

    def send_extra_req(self, ctx):
        while True:
            self.next_replica_idx = (self.next_replica_idx + 1) % len(self.nodes)
            if self.next_replica_idx not in self.replica_idxs:
                break
        print('EXTRA {0} {1}->{2}'.format(self.msg_type, self.node_id, self.next_replica_idx))
        self.send_req(ctx, self.nodes[self.next_replica_idx])

    def nack(self, sender, ctx: Context):
        if self.delivered:
            return

        print('NACK {0} {1}, id={2}'.format(sender, self.msg_type, self.id))

        if sender not in self.replicas:
            self.send_extra_req(ctx)

        else:
            self.send_req(ctx, sender)
            if sender not in self.missing_replicas:
                self.missing_replicas.add(sender)
                self.send_extra_req(ctx)
                
class StorageNode(Node):
    def __init__(self, node_id: str, nodes: List[str]):
        self._id = node_id
        self._nodes = nodes
        self._data = {}

        self.req_ctr = 0
        self.requests = {}

    def on_local_message(self, msg: Message, ctx: Context):
        # Get key value.
        # Request:
        #   GET {"key": "some key", "quorum": 1-3}
        # Response:
        #   GET_RESP {"key": "some key", "value": "value for this key"}
        #   GET_RESP {"key": "some key", "value": null} - if record for this key is not found
        if msg.type == 'GET':
            key = msg['key']

            req_id = str(self._id) + ":" + str(self.req_ctr)
            self.req_ctr += 1
            self.requests[req_id] = RequestContext(ctx, self._nodes, self._id, req_id, key, None, msg['quorum'], 'GET')

        # Store (key, value) record
        # Request:
        #   PUT {"key": "some key", "value: "some value", "quorum": 1-3}
        # Response:
        #   PUT_RESP {"key": "some key", "value: "some value"}
        elif msg.type == 'PUT':
            key = msg['key']
            value = msg['value']

            req_id = str(self._id) + ":" + str(self.req_ctr)
            self.req_ctr += 1
            self.requests[req_id] = RequestContext(ctx, self._nodes, self._id, req_id, key, value, msg['quorum'], 'PUT')

        # Delete value for the key
        # Request:
        #   DELETE {"key": "some key", "quorum": 1-3}
        # Response:
        #   DELETE_RESP {"key": "some key", "value": "some value"}
        elif msg.type == 'DELETE':
            key = msg['key']

            req_id = str(self._id) + ":" + str(self.req_ctr)
            self.req_ctr += 1
            self.requests[req_id] = RequestContext(ctx, self._nodes, self._id, req_id, key, None, msg['quorum'], 'DELETE')


    def on_message(self, msg: Message, sender: str, ctx: Context):
        # Implement node-to-node communication using any message types
        if msg.type.endswith('_ACK'):
            self.requests[msg['id']].ack(ctx, msg, sender)

        if msg.type == 'GET_REQ':
            key = msg['key']
            context, value = self._data.get(key, (-1, None))
            resp = Message('GET_ACK', {
                'id': msg['id'],
                'key': key,
                'value': value,
                'context': context,
            })
            ctx.send(resp, sender)

        elif msg.type == 'PUT_REQ':
            key = msg['key']
            context = msg['context']
            value = msg['value']

            cur_context, cur_value = self._data.get(key, (-1, None))

            if is_newer(context, value, cur_context, cur_value):
                self._data[key] = (context, value)
            else:
                context = cur_context
                value = cur_value

            resp = Message('PUT_ACK', {
                'id': msg['id'],
                'key': key,
                'value': value,
                'context': context,
            })
            ctx.send(resp, sender)


        elif msg.type == 'DELETE_REQ':
            key = msg['key']
            value = None
            context = msg['context']
            cur_context, cur_value = self._data.get(key, (-1, None))

            if cur_context <= context:
                value = cur_value
                self._data[key] = (context, None)
            else:
                context = -1
                value = None
                
            resp = Message('DELETE_ACK', {
                'id': msg['id'],
                'key': key,
                'value': value,
                'context': context,
            })
            ctx.send(resp, sender)

        elif msg.type == 'REPAIR_UPDATE':
            key = msg['key']
            context = msg['context']
            value = msg['value']

            cur_context, cur_value = self._data.get(key, (-1, None))

            if is_newer(context, value, cur_context, cur_value):
                self._data[key] = (context, value)

        elif msg.type == 'REPAIR_DELETE':
            key = msg['key']
            context = msg['context']

            cur_context, cur_value = self._data.get(key, (-1, None))

            if cur_context < context:
                self._data.pop(key, (-1, None))

    def on_timer(self, timer_name: str, ctx: Context):
        msg_type, node, req_id = timer_name.split('$', maxsplit=2)
        self.requests[req_id].nack(node, ctx)
        


def get_key_replicas(key: str, node_count: int):
    replicas = []
    key_hash = int.from_bytes(hashlib.md5(key.encode('utf8')).digest(), 'little', signed=False)
    cur = key_hash % node_count
    for _ in range(3):
        replicas.append(cur)
        cur = get_next_replica(cur, node_count)
    return replicas


def get_next_replica(i, node_count: int):
    return (i + 1) % node_count
