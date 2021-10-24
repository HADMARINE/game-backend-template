import socketHandler from '@src/modules/socket';

export default {
  createTcpChannel: (
    pref: socketHandler.ChannelCreatePreferences.Tcp,
    hdlr: socketHandler.JsHandlerFunction,
  ): socketHandler.WrappedJsInterface => {
    const res = socketHandler.createTcpChannel(pref, hdlr);

    return {
      port: res.port,
      socketHandler: (event: string, data: Record<string, any>) =>
        res.socketHandler(res.interface, event, data),
    };
  },
  createUdpChannel: (
    pref: socketHandler.ChannelCreatePreferences.Tcp,
    hdlr: socketHandler.JsHandlerFunction,
  ): socketHandler.WrappedJsInterface => {
    const res = socketHandler.createUdpChannel(pref, hdlr);

    return {
      port: res.port,
      socketHandler: (event: string, data: Record<string, any>) =>
        res.socketHandler(res.interface, event, data),
    };
  },
};
