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
      name:
        process.env.NODE_ENV === "development"
          ? "next-auth.session-token"
          : "__Secure-next-auth.session-token",
      options: {
        httpOnly: false,
        sameSite: "Lax",
        path: "/",
        secure: true,
      },
    },
  },
});
