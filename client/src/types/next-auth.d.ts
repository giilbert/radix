import NextAuth from "next-auth";
import { AdapterUser as DefaultAdapterUser } from "next-auth/adapters";

declare module "next-auth/adapters" {
  interface AdapterUser extends DefaultAdapterUser {
    accessToken: string;
  }
}
