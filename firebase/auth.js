import { app } from './init.js';
import {
  getAuth,
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
  signOut,
  onAuthStateChanged,
  GoogleAuthProvider,
  GithubAuthProvider,
  signInWithPopup
} from "https://www.gstatic.com/firebasejs/12.8.0/firebase-auth.js";

const auth = getAuth(app);

export function signUp(email, password) {
  return createUserWithEmailAndPassword(auth, email, password);
}

export function signIn(email, password) {
  return signInWithEmailAndPassword(auth, email, password);
}

export function signOutUser() {
  return signOut(auth);
}

export function onAuthState(cb) {
  return onAuthStateChanged(auth, cb);
}

export function signInWithGoogle() {
  const provider = new GoogleAuthProvider();
  return signInWithPopup(auth, provider);
}

export function signInWithGithub() {
  const provider = new GithubAuthProvider();
  return signInWithPopup(auth, provider);
}

export { auth };
