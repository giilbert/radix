import NextAuth, { DefaultSession, DefaultUser } from "next-auth";
import { AdapterUser as DefaultAdapterUser } from "next-auth/adapters";

declare module "next-auth" {
  interface Session extends DefaultSession {
    user: User;
  }
  interface User extends DefaultUser {
    sessionToken: string;
  }
}

declare module "next-auth/adapters" {
  interface AdapterUser extends DefaultAdapterUser {
    accessToken: string;
  }
}
