import SEO from "../components/SEO";
import { MilestoneInsightsPanel } from "../../milestone_insights";

const demoCampaign = {
  campaignId: "demo_camp",
  campaignTitle: "Community Solar Micro-grid",
  raisedAmount: 42_000,
  goalAmount: 100_000,
  contributorCount: 24,
  historyRaisedTotals: [8_000, 18_000, 29_000, 42_000],
};

const HomePage = () => {
  return (
    <>
      <SEO />
      <main style={{ padding: "1.5rem", maxWidth: 960, margin: "0 auto" }}>
        <h1>CrowdFund</h1>
        <p style={{ color: "#4b5563", marginBottom: "1.5rem" }}>
          Live-style milestone insights for campaign dashboards and celebration flows.
        </p>
        <MilestoneInsightsPanel input={demoCampaign} />
      </main>
    </>
  );
};

export default HomePage;
