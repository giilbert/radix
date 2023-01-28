import { Problem, TestCase } from "@/types/problem";
import { PublicUser } from "@/types/user";
import { BACKEND_URL } from "@/utils/consts";
import { Enum } from "@/utils/enum";
import { Text } from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import { AxiosError } from "axios";
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
import { UhOh } from "../ui/uh-oh";

type ChatMessage = Enum<{
  Connection: {
    username: string;
  };
  Disconnection: {
    username: string;
  };
  UserChat: {
    author: PublicUser;
    content: string;
  };
  RoundBegin: null;
  RoundEnd: null;
  UserSubmitted: {
    username: string;
  };
  UserProblemCompletion: {
    username: string;
    problemIndex: number;
  };
  UserFinished: {
    username: string;
    place: number;
  };
}>;

type RoomConfig = {
  name: string;
  public: boolean;
  owner: PublicUser;
};

type ServerSentCommand = Enum<{
  ChatMessage: ChatMessage;
  ChatHistory: ChatMessage[];
  SetUsers: PublicUser[];
  SetRoomConfig: RoomConfig;
  SetProblems: Problem[] | null;
  SetTestResponse: TestResponse;
}>;

type ClientSentCommand = Enum<{
  SendChatMessage: {
    content: string;
  };
  BeginRound: null;
  SetEditorContent: { content: string };
  TestCode: {
    language: string;
    testCases: { input: string; output: string }[];
  };
  SubmitCode: {
    problemIndex: number;
    language: string;
  };
}>;

type TestStatus = Enum<{
  None: null;
  Awaiting: null;
  Response: TestResponse;
}>;

type TestResponse = Enum<{
  Error: {
    message: string;
  };
  Ran: {
    failedTests: (TestCase & { expected: string })[];
    okayTests: TestCase[];
  };
  AllTestsPassed: {
    runtime: number;
  };
}>;

export const useRoomData = create<{
  chatMessages: ChatMessage[];
  users: PublicUser[];
  roomConfig: RoomConfig | null;
  problems: Problem[] | null;
  code: Map<number, Map<string, string>>;
  currentProblemIndex: number;
  testStatus: TestStatus;
  setCurrentProblemIndex: (index: number) => void;
  addChatMessage: (message: ChatMessage) => void;
  setChatMessages: (messages: ChatMessage[]) => void;
  setUsers: (users: PublicUser[]) => void;
  setRoomConfig: (roomConfig: RoomConfig) => void;
  setProblems: (problems: Problem[] | null) => void;
  setProblemCode: (problemId: number, language: string, code: string) => void;
  setTestStatus: (status: TestStatus) => void;
}>((set) => ({
  chatMessages: [],
  users: [],
  roomConfig: null,
  problems: null,
  code: new Map(),
  currentProblemIndex: 0,
  testStatus: {
    t: "None",
    c: null,
  },
  setChatMessages: (messages: ChatMessage[]) =>
    set({
      chatMessages: messages,
    }),
  addChatMessage: (message: ChatMessage) =>
    set((old) => ({
      chatMessages: [...old.chatMessages, message],
    })),
  setUsers: (users: PublicUser[]) =>
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
  setTestStatus: (status: TestStatus) =>
    set({
      testStatus: status,
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
  const canConnectQuery = useQuery<
    {
      canConnect: boolean;
      reason: string;
    },
    AxiosError
  >([`room/${router.query.name}/can-connect`], {
    enabled: !!router.query.name,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
  });

  const addChatMessage = useRoomData((s) => s.addChatMessage);
  const setChatMessages = useRoomData((s) => s.setChatMessages);
  const setUsers = useRoomData((s) => s.setUsers);
  const setRoomConfig = useRoomData((s) => s.setRoomConfig);
  const setProblems = useRoomData((s) => s.setProblems);
  const setTestStatus = useRoomData((s) => s.setTestStatus);

  useEffect(() => {
    if (
      wsRef.current ||
      !router.query.name ||
      !canConnectQuery.data ||
      !canConnectQuery.data.canConnect
    )
      return;

    wsRef.current = new WebSocket(
      BACKEND_URL.replace("http", "ws") + `/room/${router.query.name}`
    );

    const onConnect = (e: Event) => {
      setLoading(false);
    };

    const onMessage = (e: MessageEvent) => {
      const data: ServerSentCommand = JSON.parse(e.data);

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
      } else if (data.t === "SetTestResponse") {
        setTestStatus({
          t: "Response",
          c: data.c,
        });
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
    canConnectQuery.data,
    router.query.name,
    addChatMessage,
    setChatMessages,
    setUsers,
    setRoomConfig,
    setProblems,
    setTestStatus,
  ]);

  if (canConnectQuery.status === "loading") return <Text>Loading</Text>;
  if (!canConnectQuery.data?.canConnect) {
    const reason = canConnectQuery.data?.reason || "";

    return (
      <UhOh
        code={reason === "Room does not exist." ? "NOT_FOUND" : "FORBIDDEN"}
        message={`Unable to connect: ${canConnectQuery.data?.reason}`}
      />
    );
  }

  if (closed) return <Text>Disconnected</Text>;
  if (loading) return <Text>Loading</Text>;

  return (
    <RoomContext.Provider value={wsRef.current!}>
      {children}
    </RoomContext.Provider>
  );
};
