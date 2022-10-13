from dslib import Context, Message, Node
from typing import List, Set
import random

class MessageInfo:
    def __init__(self, text):
        self.text = text
        self.delivered = False
        self.consensus = 1

class BroadcastNode(Node):
    def __init__(self, node_id: str, nodes: List[str]):
        self._id = node_id
        self._nodes = nodes
        self.seen = set()
        self.messages = dict()
        self.counter = 0
    
    def check_consensus(self, msg_hash, ctx: Context):
        # print('check:', self._id, msg_hash, self.messages[msg_hash].consensus, self.messages[msg_hash].delivered)
        if (self.messages[msg_hash].consensus >= (len(self._nodes) + 1) / 2 and not self.messages[msg_hash].delivered):
            # print('deliver', self._id, msg_hash)
            deliver_msg = Message('DELIVER', {
                'text': self.messages[msg_hash].text,
            })
            ctx.send_local(deliver_msg)
            self.messages[msg_hash].delivered = True

    def on_local_message(self, msg: Message, ctx: Context):
        if msg.type == 'SEND':
            msg_hash = self._id + ":" + str(self.counter)
            self.counter += 1
            self.seen.add(msg_hash)
            self.messages[msg_hash] = MessageInfo(msg['text'])
            # print('send', self._id, msg_hash)

            bcast_msg = Message('BCAST', {
                'text': msg['text'],
                'hash': msg_hash,
            })

            for node in self._nodes:
                if node != self._id: # we don't want to speak to ourselves, at least not when people are watching
                    # print("cast:", self._id, msg_hash, node)
                    ctx.send(bcast_msg, node)

            self.check_consensus(msg_hash, ctx)

    def on_message(self, msg: Message, sender: str, ctx: Context):
        if msg.type == 'ACK':
            self.messages[msg['hash']].consensus += 1
            # print('got_ack', self._id, msg['hash'], 'from', sender, self.messages[msg['hash']].consensus, len(self._nodes))
            self.check_consensus(msg['hash'], ctx)

        if msg.type == 'BCAST':
            # print("got:", self._id, msg['hash'])

            if msg['hash'] not in self.seen: # we don't want to broadcast msg twice
                self.seen.add(msg['hash'])
                self.messages[msg['hash']] = MessageInfo(msg['text'])

                # some sort of broadcast
                for node in self._nodes:
                    if node != self._id:
                        # print("recast:", self._id, msg['hash'], node)
                        ctx.send(msg, node)

            # print('send_ack', self._id, sender)
            # send ACK
            ack_msg = Message('ACK', {
                'hash': msg['hash'],
            })
            ctx.send(ack_msg, sender)
                        
            self.check_consensus(msg['hash'], ctx)


    def on_timer(self, timer_name: str, ctx: Context):
        pass
