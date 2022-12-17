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
  RoundBegin: null;
  RoundEnd: null;
}>;

type RoomUser = {
  id: string;
  name: string;
  image: string;
};

type RoomConfig = {
  name: string;
  public: boolean;
  owner: RoomUser;
};

type BoilerplateCode = {
  python: string;
  javascript: string;
};

type Problem = {
  id: string;
  title: string;
  description: string;
  boilerplateCode: BoilerplateCode;
  defaultTestCase: {
    input: string;
    output: string;
  };
};

type ServerSentCommand = Enum<{
  ChatMessage: ChatMessage;
  ChatHistory: ChatMessage[];
  SetUsers: RoomUser[];
  SetRoomConfig: RoomConfig;
  SetProblems: Problem[] | null;
}>;

type ClientSentCommand = Enum<{
  SendChatMessage: {
    content: string;
  };
  BeginRound: null;
  SetEditorContent: { content: string; questionId: string };
  TestCode: { customTestCase: { input: string; output: string } | null };
  SubmitCode: null;
}>;

export const useRoomData = create<{
  chatMessages: ChatMessage[];
  users: RoomUser[];
  roomConfig: RoomConfig | null;
  problems: Problem[] | null;
  code: Map<number, Map<string, string>>;
  currentProblemIndex: number;
  setCurrentProblemIndex: (index: number) => void;
  addChatMessage: (message: ChatMessage) => void;
  setChatMessages: (messages: ChatMessage[]) => void;
  setUsers: (users: RoomUser[]) => void;
  setRoomConfig: (roomConfig: RoomConfig) => void;
  setProblems: (problems: Problem[] | null) => void;
  setProblemCode: (problemId: number, language: string, code: string) => void;
}>((set) => ({
  chatMessages: [],
  users: [],
  roomConfig: null,
  problems: null,
  code: new Map(),
  currentProblemIndex: 0,
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
  setRoomConfig: (roomConfig: RoomConfig) => set({ roomConfig }),
  setProblems: (problems: Problem[] | null) => {
    const code: Map<number, Map<string, string>> = new Map();
    problems?.forEach((problem, i) => {
      let map = new Map();
      map.set("python", problem.boilerplateCode.python);
      map.set("javascript", problem.boilerplateCode.javascript);
      code.set(i, map);
    });

    set({ problems, code });
  },
  setProblemCode: (problemId: number, language: string, code: string) =>
    set((before) => {
      let problem = before.code.get(problemId);
      if (!problem)
        return {
          code: before.code,
        };

      problem.set(language, code);

      return {
        code: before.code,
      };
    }),
  setCurrentProblemIndex: (index: number) =>
    set({ currentProblemIndex: index }),
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
  const setRoomConfig = useRoomData((s) => s.setRoomConfig);
  const setProblems = useRoomData((s) => s.setProblems);

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
      } else if (data.t === "SetRoomConfig") {
        setRoomConfig(data.c);
      } else if (data.t === "SetProblems") {
        setProblems(data.c);
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
  }, [
    router.query.name,
    addChatMessage,
    setChatMessages,
    setUsers,
    setRoomConfig,
    setProblems,
  ]);

  if (closed) return <Text>Disconnected</Text>;
  if (loading) return <Text>Loading</Text>;

  return (
    <RoomContext.Provider value={wsRef.current!}>
      {children}
    </RoomContext.Provider>
  );
};
