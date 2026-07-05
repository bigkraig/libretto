import next from "eslint-config-next/core-web-vitals";

// Flat config for ESLint 9 (Next 16 removed `next lint`; run `eslint .`).
const config = [
  { ignores: [".next/**", "node_modules/**", "next-env.d.ts"] },
  ...next,
  {
    rules: {
      // React-Compiler-oriented rules new in react-hooks v6 (bundled by
      // eslint-config-next 16). They flag existing, working patterns here, so keep
      // them visible as warnings rather than blocking the lint on the upgrade.
      "react-hooks/set-state-in-effect": "warn",
      "react-hooks/immutability": "warn",
    },
  },
];

export default config;
