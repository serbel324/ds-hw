from dslib import Context, Message, Node
from typing import List


class BroadcastNode(Node):
    def __init__(self, node_id: str, nodes: List[str]):
        self._id = node_id
        self._nodes = nodes

    def on_local_message(self, msg: Message, ctx: Context):
        if msg.type == 'SEND':
            bcast_msg = Message('BCAST', {
                'message': msg['message']
            })
            # best-effort broadcast
            for node in self._nodes:
                ctx.send(bcast_msg, node)

    def on_message(self, msg: Message, sender: str, ctx: Context):
        if msg.type == 'BCAST':
            # deliver message to the local user
            deliver_msg = Message('DELIVER', {
                'message': msg['message']
            })
            ctx.send_local(deliver_msg)

    def on_timer(self, timer_id: str, ctx: Context):
        pass