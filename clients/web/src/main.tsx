import ReactDOM from 'react-dom/client'
import { RouterProvider, createRouter } from '@tanstack/react-router'
import { MsalProvider } from '@azure/msal-react'
import { msalInstance } from '#/auth/msalConfig'
import { routeTree } from './routeTree.gen'

const router = createRouter({
  routeTree,
  defaultPreload: 'intent',
  scrollRestoration: true,
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

const rootElement = document.getElementById('app')!

// initialize() and handleRedirectPromise() must both complete before any
// MSAL hook (useMsal, etc.) is used — handleRedirectPromise() is what
// actually processes the auth code after loginRedirect() sends the user
// back here.
await msalInstance.initialize()
await msalInstance.handleRedirectPromise()

if (!rootElement.innerHTML) {
  const root = ReactDOM.createRoot(rootElement)
  root.render(
    <MsalProvider instance={msalInstance}>
      <RouterProvider router={router} />
    </MsalProvider>,
  )
}
