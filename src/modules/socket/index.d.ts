export type BoxedJsInterface = unknown;
export type JsHandlerFunction = (
  event: string,
  data: Record<string, any>,
) => void;
export interface JsInterface {
  port: number;
  socketHandler: (
    interface: BoxedJsInterface,
    data: Record<string, any>,
  ) => void;
}

export namespace ChannelCreatePreferences {
  interface Tcp {
    deleteClientWhenClosed: boolean;
    concurrent: boolean;
  }

  interface Udp {
    deleteClientWhenClosed: boolean;
  }
}

export function createTcpChannel(
  pref: ChannelCreatePreferences.Tcp,
  handler: JsHandlerFunction,
): JsInterface;
export function createUdpChannel(
  pref: ChannelCreatePreferences.Udp,
  handler: JsHandlerFunction,
): JsInterface;
