import SEO from "../components/SEO";
import GlobalErrorBoundary from "../components/frontend_global_error";

function MyApp({ Component, pageProps }) {
  return (
    <GlobalErrorBoundary>
      <SEO />
      <Component {...pageProps} />
    </GlobalErrorBoundary>
﻿import "../styles/global.css";
import SEO from "../components/SEO";
﻿import SEO from "../components/SEO";
import GlobalErrorBoundary from "../components/frontend_global_error";

function MyApp({ Component, pageProps }) {
  return (
    <GlobalErrorBoundary>
      <SEO />
      <Component {...pageProps} />
    </GlobalErrorBoundary>
  );
}

export default MyApp;
