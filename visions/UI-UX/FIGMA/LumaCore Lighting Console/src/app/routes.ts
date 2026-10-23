import { createBrowserRouter } from "react-router";
import Root from "./pages/Root";

export const router = createBrowserRouter([
  {
    path: "/",
    Component: Root,
  },
  {
    path: "*",
    Component: Root,
  },
]);