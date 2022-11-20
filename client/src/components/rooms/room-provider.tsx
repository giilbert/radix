import { BACKEND_URL } from "@/utils/consts";
import { Text } from "@chakra-ui/react";
import { useRouter } from "next/router";
import {
  createContext,
  PropsWithChildren,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

export interface RoomData {}

const RoomContext = createContext<RoomData | null>(null);

export const RoomProvider: React.FC<
  PropsWithChildren<Record<never, never>>
> = ({ children }) => {
  const router = useRouter();
  const wsRef = useRef<WebSocket>();
  const [loading, setLoading] = useState(true);
  const [closed, setClosed] = useState(false);

  useLayoutEffect(() => {
    if (wsRef.current || !router.query.name) return;

    console.log("constructing websocket", wsRef.current);
    wsRef.current = new WebSocket(
      BACKEND_URL.replace("http", "ws") + `/room/${router.query.name}`
    );

    const onConnect = (e: Event) => {
      setLoading(false);
    };

    const onMessage = (e: MessageEvent) => {
      console.log(e.data);
    };

    const onError = (e: Event) => {
      console.error(e);
    };

    const onClose = (e: CloseEvent) => {
      setClosed(true);
    };

    wsRef.current.addEventListener("open", onConnect);
    wsRef.current.addEventListener("message", onMessage);
    wsRef.current.addEventListener("error", onError);
    wsRef.current.addEventListener("close", onClose);

    return () => {
      wsRef.current?.removeEventListener("open", onConnect);
      wsRef.current?.removeEventListener("message", onMessage);
      wsRef.current?.removeEventListener("error", onError);
      wsRef.current?.removeEventListener("close", onClose);

      wsRef.current?.close();
      wsRef.current = undefined;
    };
  }, [router.query.name]);

  if (closed) return <Text>Disconnected</Text>;
  if (loading) return <Text>Loading</Text>;

  return <RoomContext.Provider value={null}>{children}</RoomContext.Provider>;
};
