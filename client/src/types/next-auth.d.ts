import NextAuth, { DefaultSession, DefaultUser } from "next-auth";
import { AdapterUser as DefaultAdapterUser } from "next-auth/adapters";

declare module "next-auth" {
  interface Session extends DefaultSession {
    user: User;
  }
}

declare module "next-auth/adapters" {
  interface AdapterUser extends DefaultAdapterUser {
    accessToken: string;
  }
}
