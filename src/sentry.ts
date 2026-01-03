import * as Sentry from "@sentry/react";

Sentry.init({
  dsn: "https://0342320e0cdcaa1b7737ac5ea69caad5@o4507499424186368.ingest.us.sentry.io/4510631808532480",
  integrations: [
    Sentry.browserTracingIntegration(),
  ],
  // Performance monitoring - capture 10% of transactions
  tracesSampleRate: 0.1,
});

export { Sentry };
