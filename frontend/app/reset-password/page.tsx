import { Suspense } from "react";
import ResetPasswordPage from "./client";

export default function Page() {
  return (
      <Suspense fallback={<div>Loading reset page...</div>}>
        <ResetPasswordPage />
      </Suspense>
  );
}
