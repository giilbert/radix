import { BACKEND_URL } from "@/utils/consts";
import { Enum } from "@/utils/enum";
import { Text } from "@chakra-ui/react";
import { useRouter } from "next/router";
import {
  createContext,
  PropsWithChildren,
  useEffect,
  useRef,
  useState,
} from "react";
import create from "zustand";

type ChatMessage = Enum<{
  Connection: {
    username: string;
  };
  Disconnection: {
    username: string;
  };
}>;

type Command = Enum<{
  ChatMessage: ChatMessage;
  ChatHistory: ChatMessage[];
}>;

export const useRoomData = create<{
  chatMessages: ChatMessage[];
  addChatMessage: (message: ChatMessage) => void;
  setChatMessages: (messages: ChatMessage[]) => void;
}>((set) => ({
  chatMessages: [],
  setChatMessages: (messages: ChatMessage[]) =>
    set({
      chatMessages: messages,
    }),
  addChatMessage: (message: ChatMessage) =>
    set((old) => ({
      chatMessages: [...old.chatMessages, message],
    })),
}));

const RoomContext = createContext<WebSocket | null>(null);

export const RoomProvider: React.FC<
  PropsWithChildren<Record<never, never>>
> = ({ children }) => {
  const router = useRouter();
  const wsRef = useRef<WebSocket>();
  const [loading, setLoading] = useState(true);
  const [closed, setClosed] = useState(false);
  const addChatMessage = useRoomData((s) => s.addChatMessage);
  const setChatMessages = useRoomData((s) => s.setChatMessages);

  useEffect(() => {
    if (wsRef.current || !router.query.name) return;

    console.log("constructing websocket", wsRef.current);
    wsRef.current = new WebSocket(
      BACKEND_URL.replace("http", "ws") + `/room/${router.query.name}`
    );

    const onConnect = (e: Event) => {
      setLoading(false);
    };

    const onMessage = (e: MessageEvent) => {
      const data: Command = JSON.parse(e.data);

      console.log(data);
      if (data.t === "ChatMessage") {
        addChatMessage(data.c);
      } else if (data.t === "ChatHistory") {
        setChatMessages(data.c.reverse());
      }
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
  }, [router.query.name, addChatMessage, setChatMessages]);

  if (closed) return <Text>Disconnected</Text>;
  if (loading) return <Text>Loading</Text>;

  return (
    <RoomContext.Provider value={wsRef.current!}>
      {children}
    </RoomContext.Provider>
  );
};
