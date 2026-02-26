import Head from "next/head";

interface SEOProps {
  title?: string;
  description?: string;
  canonical?: string;
  image?: string;
}

const DEFAULT_TITLE = "CrowdFund - Decentralized Crowdfunding on Stellar";
const DEFAULT_DESCRIPTION =
  "Launch and support campaigns on a transparent, decentralized crowdfunding platform built on the Stellar network using Soroban smart contracts.";
const DEFAULT_IMAGE = "/og-image.jpg";
const SITE_URL = "https://your-crowdfund-app.com";

const SEO = ({
  title = DEFAULT_TITLE,
  description = DEFAULT_DESCRIPTION,
  canonical,
  image = DEFAULT_IMAGE,
}: SEOProps) => {
  const fullTitle = title === DEFAULT_TITLE ? title : `${title} | CrowdFund`;

  const canonicalUrl = canonical ? `${SITE_URL}${canonical}` : SITE_URL;

  return (
    <Head>
      {/* Primary Meta Tags */}
      <title>{fullTitle}</title>
      <meta name="description" content={description} />

      {/* Viewport */}
      <meta name="viewport" content="width=device-width, initial-scale=1.0" />

      {/* Canonical */}
      <link rel="canonical" href={canonicalUrl} />

      {/* Charset and language */}
      <meta charSet="UTF-8" />
      <meta httpEquiv="X-UA-Compatible" content="IE=edge" />

      {/* Robots */}
      <meta name="robots" content="index, follow" />

      {/* Open Graph */}
      <meta property="og:type" content="website" />
      <meta property="og:url" content={canonicalUrl} />
      <meta property="og:title" content={fullTitle} />
      <meta property="og:description" content={description} />
      <meta property="og:image" content={`${SITE_URL}${image}`} />
      <meta property="og:image:width" content="1200" />
      <meta property="og:image:height" content="630" />
      <meta property="og:image:alt" content="CrowdFund - Decentralized Crowdfunding on Stellar" />

      {/* Twitter Card */}
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:title" content={fullTitle} />
      <meta name="twitter:description" content={description} />
      <meta name="twitter:image" content={`${SITE_URL}${image}`} />

      {/* Favicon */}
      <link rel="icon" href="/favicon.ico" />
      <link rel="icon" type="image/svg+xml" href="/favicon-32x32.svg" sizes="32x32" />
      <link rel="icon" type="image/svg+xml" href="/favicon-64x64.svg" sizes="64x64" />
      <link rel="apple-touch-icon" href="/apple-touch-icon.svg" />

      {/* Theme color */}
      <meta name="theme-color" content="#4f46e5" />
    </Head>
  );
};

export default SEO;
