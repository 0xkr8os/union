import Navigation from "./navigation.svelte"

const routes = {
  home: { draft: false, path: "/" },
  send: { draft: false, path: "/send" },
  faucet: { draft: false, path: "/faucet" },
  transfers: { draft: true, path: "/transfers" }
} as const

export { Navigation, routes }