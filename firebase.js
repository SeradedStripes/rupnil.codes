
// Compatibility wrapper: lazy-proxy to new modular files

let _auth, _db, _analytics;
const loadAuth = async () => _auth || (_auth = await import('./firebase/auth.js'));
const loadDb = async () => _db || (_db = await import('./firebase/db.js'));
const loadAnalytics = async () => _analytics || (_analytics = await import('./firebase/analytics.js'));

export async function signUp(...args) { const m = await loadAuth(); return m.signUp(...args); }
export async function signIn(...args) { const m = await loadAuth(); return m.signIn(...args); }
export async function signOutUser(...args) { const m = await loadAuth(); return m.signOutUser(...args); }
export async function onAuthState(...args) { const m = await loadAuth(); return m.onAuthState(...args); }
export async function signInWithGoogle(...args) { const m = await loadAuth(); return m.signInWithGoogle(...args); }
export async function signInWithGithub(...args) { const m = await loadAuth(); return m.signInWithGithub(...args); }
export async function requestAlias(...args) { const m = await loadDb(); return m.requestAlias(...args); }
export async function checkAliasStatus(...args) { const m = await loadDb(); return m.checkAliasStatus(...args); }
export async function ensureUserDoc(...args) { const m = await loadDb(); return m.ensureUserDoc(...args); }
export async function initAnalytics(...args) { const m = await loadAnalytics(); return m.initAnalytics(...args); }

// Deprecated: for compatibility, export getters
export async function getAuth() { const m = await loadAuth(); return m.auth; }