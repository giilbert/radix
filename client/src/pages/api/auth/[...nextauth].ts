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
  callbacks: {
    async session({ session, user }) {
      session.user.id = user.id;
      return session;
    },
  },
  adapter: CustomAdapter(),
  cookies: {
    sessionToken: {
      name: "next-auth.session-token",
      options: {
        httpOnly: false,
        sameSite: "None",
        path: "/",
        secure: true,
        domain:
          process.env.NODE_ENV === "production" ? "gilbertz.tech" : undefined,
      },
    },
  },
});
