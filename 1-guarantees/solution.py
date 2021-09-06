from dslib import Context, Message, Node


class Sender(Node):
    def __init__(self, node_id: str, receiver_id: str):
        self._id = node_id
        self._receiver = receiver_id

    def on_local_message(self, msg: Message, ctx: Context):
        # receive info for delivery from local user
        if msg.type == 'INFO-1':
            # deliver this info at most once
            pass
        elif msg.type == 'INFO-2':
            # deliver this info at least once
            pass
        elif msg.type == 'INFO-3':
            # deliver this info exactly once
            pass
        elif msg.type == 'INFO-4':
            # deliver these info exactly once and keeping their order
            pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver here
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


class Receiver(Node):
    def __init__(self, node_id: str):
        self._id = node_id

    def on_local_message(self, msg: Message, ctx: Context):
        # not used in this task
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver
        # deliver info to local user with ctx.send_local()
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass
