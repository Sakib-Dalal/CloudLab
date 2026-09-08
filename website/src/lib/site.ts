import * as publicEnv from "$env/static/public";

export const repository = "https://github.com/Sakib-Dalal/CloudLab";
const configuredUrl =
  Object.entries(publicEnv).find(([key]) => key === "PUBLIC_SITE_URL")?.[1] ||
  "https://cloudlab-alpha.vercel.app";
export const siteUrl = new URL(String(configuredUrl)).origin;
