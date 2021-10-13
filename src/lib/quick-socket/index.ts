import socketHandler from '@src/modules/socket';

export default {
  createTcpChannel: (
    pref: socketHandler.ChannelCreatePreferences.Tcp,
    hdlr: socketHandler.JsHandlerFunction,
  ): socketHandler.WrappedJsInterface => {
    const res = socketHandler.createTcpChannel(pref, hdlr);

    return {
      port: res.port,
      socketHandler: (data: Record<string, any>) =>
        res.socketHandler(res.interface, data),
    };
  },
  createUdpChannel: (
    pref: socketHandler.ChannelCreatePreferences.Tcp,
    hdlr: socketHandler.JsHandlerFunction,
  ): socketHandler.WrappedJsInterface => {
    const res = socketHandler.createUdpChannel(pref, hdlr);

    return {
      port: res.port,
      socketHandler: (data: Record<string, any>) =>
        res.socketHandler(res.interface, data),
    };
  },
};
