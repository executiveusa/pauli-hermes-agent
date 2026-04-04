/**
 * Edge-safe auth configuration — shared between middleware and server.
 * Contains only JWT/session/callbacks/pages config. No providers, no DB imports.
 * Both instances use the same AUTH_SECRET for JWT signing/verification.
 *
 * Official pattern: https://authjs.dev/guides/edge-compatibility
 */
export const authConfig = {
  providers: [],
  session: { strategy: 'jwt' },
  pages: { signIn: '/login' },
  callbacks: {
    jwt({ token, user }) {
      if (user) {
        token.role = user.role;
      }
      return token;
    },
    session({ session, token }) {
      if (session.user) {
        session.user.id = token.sub;
        session.user.role = token.role;
      }
      return session;
    },
  },
};
