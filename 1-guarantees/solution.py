from dslib import Context, Message, Node


# AT MOST ONCE ---------------------------------------------------------------------------------------------------------

class AtMostOnceSender(Node):
    def __init__(self, node_id: str, receiver_id: str):
        self._id = node_id
        self._receiver = receiver_id

    def on_local_message(self, msg: Message, ctx: Context):
        # receive message for delivery from local user
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver here
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


class AtMostOnceReceiver(Node):
    def __init__(self, node_id: str):
        self._id = node_id

    def on_local_message(self, msg: Message, ctx: Context):
        # not used in this task
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver
        # deliver message to local user with ctx.send_local()
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


# AT LEAST ONCE --------------------------------------------------------------------------------------------------------

class AtLeastOnceSender(Node):
    def __init__(self, node_id: str, receiver_id: str):
        self._id = node_id
        self._receiver = receiver_id

    def on_local_message(self, msg: Message, ctx: Context):
        # receive message for delivery from local user
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver here
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


class AtLeastOnceReceiver(Node):
    def __init__(self, node_id: str):
        self._id = node_id

    def on_local_message(self, msg: Message, ctx: Context):
        # not used in this task
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver
        # deliver message to local user with ctx.send_local()
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


# EXACTLY ONCE ---------------------------------------------------------------------------------------------------------

class ExactlyOnceSender(Node):
    def __init__(self, node_id: str, receiver_id: str):
        self._id = node_id
        self._receiver = receiver_id

    def on_local_message(self, msg: Message, ctx: Context):
        # receive message for delivery from local user
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver here
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


class ExactlyOnceReceiver(Node):
    def __init__(self, node_id: str):
        self._id = node_id

    def on_local_message(self, msg: Message, ctx: Context):
        # not used in this task
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver
        # deliver message to local user with ctx.send_local()
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


# EXACTLY ONCE + ORDERED -----------------------------------------------------------------------------------------------

class ExactlyOnceOrderedSender(Node):
    def __init__(self, node_id: str, receiver_id: str):
        self._id = node_id
        self._receiver = receiver_id

    def on_local_message(self, msg: Message, ctx: Context):
        # receive message for delivery from local user
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver here
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass


class ExactlyOnceOrderedReceiver(Node):
    def __init__(self, node_id: str):
        self._id = node_id

    def on_local_message(self, msg: Message, ctx: Context):
        # not used in this task
        pass

    def on_message(self, msg: Message, sender: str, ctx: Context):
        # process messages from receiver
        # deliver message to local user with ctx.send_local()
        pass

    def on_timer(self, timer_id: str, ctx: Context):
        # process fired timers here
        pass
