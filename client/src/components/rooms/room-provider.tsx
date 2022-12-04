import { BACKEND_URL } from "@/utils/consts";
import { Enum } from "@/utils/enum";
import { Text } from "@chakra-ui/react";
import { useRouter } from "next/router";
import {
  createContext,
  PropsWithChildren,
  useCallback,
  useContext,
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
  UserChat: {
    author: RoomUser;
    content: string;
  };
}>;

type RoomUser = {
  id: string;
  name: string;
  image: string;
};

type ServerSentCommand = Enum<{
  ChatMessage: ChatMessage;
  ChatHistory: ChatMessage[];
  SetUsers: RoomUser[];
}>;

type ClientSentCommand = Enum<{
  SendChatMessage: {
    content: string;
  };
}>;

export const useRoomData = create<{
  chatMessages: ChatMessage[];
  users: RoomUser[];
  addChatMessage: (message: ChatMessage) => void;
  setChatMessages: (messages: ChatMessage[]) => void;
  setUsers: (users: RoomUser[]) => void;
}>((set) => ({
  chatMessages: [],
  users: [],
  setChatMessages: (messages: ChatMessage[]) =>
    set({
      chatMessages: messages,
    }),
  addChatMessage: (message: ChatMessage) =>
    set((old) => ({
      chatMessages: [...old.chatMessages, message],
    })),
  setUsers: (users: RoomUser[]) =>
    set({
      users,
    }),
}));

export const useRoom = () => {
  const ws = useContext(RoomContext);
  if (!ws) throw "Using RoomContext before initialization.";
  const sendCommand = useCallback(
    (command: ClientSentCommand) => {
      ws.send(JSON.stringify(command));
    },
    [ws]
  );

  return { sendCommand };
};

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
  const setUsers = useRoomData((s) => s.setUsers);

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
      const data: ServerSentCommand = JSON.parse(e.data);

      console.log(data);
      if (data.t === "ChatMessage") {
        addChatMessage(data.c);
      } else if (data.t === "ChatHistory") {
        setChatMessages(data.c);
      } else if (data.t === "SetUsers") {
        setUsers(data.c);
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
