import { app } from './init.js';

export async function initAnalytics() {
  try {
    const { getAnalytics } = await import("https://www.gstatic.com/firebasejs/12.8.0/firebase-analytics.js");
    return getAnalytics(app);
  } catch (err) {
    console.warn('Analytics failed to load:', err);
    return null;
  }
}
