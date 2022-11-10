import { axios } from "@/utils/axios";
import { Adapter } from "next-auth/adapters";

export function CustomAdapter(): Adapter {
  return {
    createUser: async (user) => {
      console.log("createUser", user);
      const res = await axios.post("auth/user", {
        ...user,
        accounts: [],
        sessions: [],
      });
      return res.data;
    },

    getUser: async (id) => {
      console.log("getUser", id);
      const res = await axios.post("auth/user/" + id);
      if (res.status === 200) return res.data;
      console.error(res.data);
      return null;
    },

    getUserByEmail: async (email) => {
      console.log("getUserByEmail", email);
      const res = await axios.get("auth/user-email/" + email);
      if (res.status === 200) return res.data;
      console.error(res.data);
      return null;
    },

    getUserByAccount: async (account) => {
      console.log("getUserByAccount", account);
      const res = await axios.get(
        "auth/user-account/" +
          account.provider +
          "/" +
          account.providerAccountId
      );
      if (res.status === 200) return res.data;
      console.error(res.data);

      return null;
    },

    linkAccount: async (acc) => {
      const data = {
        userId: acc.userId,
        provider: acc.provider,
        providerAccountId: acc.providerAccountId,
        providerType: acc.type,
        accessToken: acc.access_token,
        expiresAt: acc.expires_at,
        scope: acc.scope,
        tokenType: acc.token_type,
        idToken: acc.id_token,
      };

      console.log("linkAccount", JSON.stringify(data));
      await axios.post("auth/link-account", data);
    },

    createSession: async (session) => {
      console.log("createSession", session);
      await axios.post("auth/session", session);
      return session;
    },

    getSessionAndUser: async (sessionToken: string) => {
      console.log("getSessionAndUser", sessionToken);
      const res = await axios.get("auth/session/" + sessionToken);
      if (res.status === 404) return null;
      if (!res.data) throw "Error fetching session and user";

      return {
        user: res.data.user,
        session: {
          ...res.data.session,
          expires: new Date(res.data.session.expires),
        },
      };
    },

    deleteSession: async (sessionToken) => {
      console.log("deleteSession", sessionToken);
      const res = await axios.delete("auth/session/" + sessionToken);
      if (res.status !== 200) throw "Error deleting session";
    },

    updateUser: async (user) => {
      console.log("updateUser", user);
      throw "unimplemented";
    },

    updateSession: async () => {
      throw "unimplemented";
    },
  };
}
