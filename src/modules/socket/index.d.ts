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
  interface: BoxedJsInterface;
}

export interface WrappedJsInterface {
  port: number;
  socketHandler: (data: Record<string, any>) => void;
}

export namespace ChannelCreatePreferences {
  interface Tcp {
    deleteClientWhenClosed: boolean;
    concurrent: boolean;
    preset: string;
  }

  interface Udp {
    deleteClientWhenClosed: boolean;
    preset: string;
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
