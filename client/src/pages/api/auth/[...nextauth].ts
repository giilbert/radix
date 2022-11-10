import NextAuth from "next-auth";
import { GoogleProvider } from "@/lib/auth/provider";
import { CustomAdapter } from "@/lib/auth/adapter";

export default NextAuth({
  providers: [
    GoogleProvider({
      clientId: process.env.GOOGLE_CLIENT_ID as string,
      clientSecret: process.env.GOOGLE_CLIENT_SECRET as string,
    }),
  ],
  adapter: CustomAdapter(),
});
