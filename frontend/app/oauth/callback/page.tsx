import { Suspense } from "react";
import OAuthCallbackPage from "./client";

export default function Page() {
  return (
      <Suspense fallback={<div>Loading OAuth callback...</div>}>
        <OAuthCallbackPage />
      </Suspense>
  );
}
