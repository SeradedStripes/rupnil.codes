import { onAuthState, signOutUser } from '/firebase.js';

export function initHeader(root = document) {
  const toggle = root.querySelector('.header-section__toggle');
  const links = root.querySelector('.header-section__links');
  if (!toggle || !links) return;

  const setExpanded = (v) => toggle.setAttribute('aria-expanded', String(v));

  toggle.addEventListener('click', () => {
    const expanded = toggle.getAttribute('aria-expanded') === 'true';
    setExpanded(!expanded);
    links.classList.toggle('active');
  });

  links.querySelectorAll('a').forEach(a => a.addEventListener('click', () => {
    links.classList.remove('active');
    setExpanded(false);
  }));

  document.addEventListener('click', (e) => {
    if (!e.target.closest('.header-section')) {
      links.classList.remove('active');
      setExpanded(false);
    }
  });
}

export function initHeaderAccount(root = document) {
  const account = root.querySelector('#header-account');
  if (!account) return;

  onAuthState((user) => {
    if (!account) return;
    if (!user) {
      account.innerHTML = '<a href="/account/login.html" class="header-section__btn header-signin">Sign in</a>';
    } else {
      account.innerHTML = `<a href="/account/dashboard.html" class="header-section__btn header-account-btn" title="${user.email}"><span aria-hidden="true"><img width="24" height="24" src="${user.photoURL}" alt="User Avatar"></span></a>`;
      const signoutBtn = account.querySelector('.header-signout');
      if (signoutBtn) {
        signoutBtn.addEventListener('click', async () => {
          try {
            await signOutUser();
            window.location.href = '/';
          } catch (err) {
            alert('Sign out failed: ' + (err.message || err));
          }
        });
      }
    }
  });
}

export async function loadHeader() {
  const container = document.getElementById('site-header');
  if (!container) return;

  if (container.innerHTML.trim() === '') {
    try {
      const res = await fetch('/header.html');
      if (!res.ok) throw new Error('Failed to fetch /header.html: ' + res.status);
      const html = await res.text();
      container.innerHTML = html;
    } catch (err) {
      console.error('loadHeader error:', err);
      return;
    }
  }

  initHeader(container);
  initHeaderAccount(container);
}

loadHeader();
