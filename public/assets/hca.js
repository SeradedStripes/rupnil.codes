// Default API base logic:
// - If window.API_BASE is set, use that
// - If running on localhost, use the current origin
// - Otherwise default to the production API origin
function normalizeOrigin(orig) {
  return (orig || '').replace(/\/+$/, '');
}

const DEFAULT_API_BASE = (window.location && (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'))
  ? `${window.location.protocol}//${window.location.host}`
  : 'https://api.rupnil.codes';

let API_BASE = normalizeOrigin(window.API_BASE) || DEFAULT_API_BASE;

export function openHcaPopup() {
  const url = `${API_BASE}/auth/hack_club`;
  const width = 500; const height = 700;
  const left = Math.round(window.screenX + (window.outerWidth - width) / 2);
  const top = Math.round(window.screenY + (window.outerHeight - height) / 2);
  const popup = window.open(url, 'hca_auth', `width=${width},height=${height},left=${left},top=${top}`);

  return new Promise((resolve, reject) => {
    if (!popup) return reject(new Error('Unable to open popup'));
    let settled = false;

    function onMessage(e) {
      const data = e.data;
      if (data && data.jwt) {
        settled = true;
        window.removeEventListener('message', onMessage);
        try { popup.close(); } catch (e) {}
        resolve(data);
      }
    }

    window.addEventListener('message', onMessage);

    const timer = setInterval(() => {
      if (popup.closed && !settled) {
        clearInterval(timer);
        window.removeEventListener('message', onMessage);
        reject(new Error('Authentication popup closed'));
      }
    }, 500);

    setTimeout(() => {
      if (!settled) {
        clearInterval(timer);
        window.removeEventListener('message', onMessage);
        try { popup.close(); } catch (e) {}
        reject(new Error('Authentication timed out'));
      }
    }, 5 * 60 * 1000);
  });
}

export function saveTokens(jwt, refreshToken) {
  localStorage.setItem('rp_jwt', jwt);
  localStorage.setItem('rp_rt', refreshToken);
}

export function clearTokens() {
  localStorage.removeItem('rp_jwt');
  localStorage.removeItem('rp_rt');
}

export function getJwt() { return localStorage.getItem('rp_jwt'); }
export function getRefreshToken() { return localStorage.getItem('rp_rt'); }

export async function apiFetch(path, opts = {}) {
  const jwt = getJwt();
  const headers = opts.headers ? { ...opts.headers } : {};
  if (jwt) headers['Authorization'] = `Bearer ${jwt}`;
  if (opts.json) {
    headers['Content-Type'] = 'application/json';
    opts.body = JSON.stringify(opts.json);
    delete opts.json;
  }
  const res = await fetch(API_BASE + path, { ...opts, headers });
  if (!res.ok) {
    const text = await res.text().catch(() => '');
    const err = new Error(text || res.statusText);
    err.status = res.status;
    throw err;
  }
  const ct = res.headers.get('content-type') || '';
  if (ct.includes('application/json')) return res.json();
  return res.text();
}
