import { Adapter } from "next-auth/adapters";

export function CustomAdapter(): Adapter {
  return {
    createSession: async (session) => {
      return {
        expires: new Date(),
        sessionToken: "asdsd",
        userId: "asdsd",
      };
    },

    createUser: async (user) => {
      return {
        email: "email",
        emailVerified: null,
        id: "Asdsad",
        image: "asds",
        name: "asd",
      };
    },

    deleteSession: async (session) => {
      return {};
    },

    getUser: (id) => {
      return null;
    },

    /*

    createUser: (user: Omit<AdapterUser, "id">) => Awaitable<AdapterUser>;
    getUser: (id: string) => Awaitable<AdapterUser | null>;
    getUserByEmail: (email: string) => Awaitable<AdapterUser | null>;
    getUserByAccount: (providerAccountId: Pick<AdapterAccount, "provider" | "providerAccountId">) => Awaitable<AdapterUser | null>;
    updateUser: (user: Partial<AdapterUser>) => Awaitable<AdapterUser>;
    deleteUser?: (userId: string) => Promise<void> | Awaitable<AdapterUser | null | undefined>;
    linkAccount: (account: AdapterAccount) => Promise<void> | Awaitable<AdapterAccount | null | undefined>;
    unlinkAccount?: (providerAccountId: Pick<AdapterAccount, "provider" | "providerAccountId">) => Promise<void> | Awaitable<AdapterAccount | undefined>;
    createSession: (session: {
        sessionToken: string;
        userId: string;
        expires: Date;
    }) => Awaitable<AdapterSession>;
    getSessionAndUser: (sessionToken: string) => Awaitable<{
        session: AdapterSession;
        user: AdapterUser;
    } | null>;
    updateSession: (session: Partial<AdapterSession> & Pick<AdapterSession, "sessionToken">) => Awaitable<AdapterSession | null | undefined>;
    deleteSession: (sessionToken: string) => Promise<void> | Awaitable<AdapterSession | null | undefined>;
    createVerificationToken?: (verificationToken: VerificationToken) => Awaitable<VerificationToken | null | undefined>;
    useVerificationToken?: (params: {
        identifier: string;
        token: string;
    }) => Awaitable<VerificationToken | null>;
*/
  };
}
