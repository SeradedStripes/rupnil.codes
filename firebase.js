
// Import the functions you need from the SDKs you need
import { initializeApp } from "https://www.gstatic.com/firebasejs/12.8.0/firebase-app.js";
import { getAnalytics } from "https://www.gstatic.com/firebasejs/12.8.0/firebase-analytics.js";
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
import { getFirestore, doc, setDoc, getDoc, serverTimestamp } from "https://www.gstatic.com/firebasejs/12.8.0/firebase-firestore.js";

// Your web app's Firebase configuration
// For Firebase JS SDK v7.20.0 and later, measurementId is optional
const firebaseConfig = {
  apiKey: "AIzaSyD39urvj-ogav5fdovwfUFnNoFQBkkw51Q",
  authDomain: "rupnil-codes.firebaseapp.com",
  projectId: "rupnil-codes",
  storageBucket: "rupnil-codes.firebasestorage.app",
  messagingSenderId: "49829826847",
  appId: "1:49829826847:web:f9e555afe7e4059d9aa02a",
  measurementId: "G-XDMV0402P9"
};

// Initialize Firebase
const app = initializeApp(firebaseConfig);
const analytics = getAnalytics(app);
const auth = getAuth(app);
const db = getFirestore(app);

export async function requestAlias(alias) {
  const user = auth.currentUser;
  if (!user) throw new Error("Not logged in");

  const aliasDoc = doc(db, "aliasRequests", user.uid);

  await setDoc(aliasDoc, {
    uid: user.uid,
    alias: alias,
    status: "pending",
    createdAt: serverTimestamp()
  });

  console.log("Alias request sent!");
} 

export async function checkAliasStatus(alias) {
  const aliasDoc = await getDoc(doc(db, "aliasRequests", alias));
  if (!aliasDoc.exists()) return null;
  return aliasDoc.data().status;
}

export async function ensureUserDoc(user) {
  const u = user || auth.currentUser;
  if (!u) throw new Error("Not logged in");

  const userRef = doc(db, "users", u.uid);
  const snap = await getDoc(userRef);
  if (!snap.exists()) {
    await setDoc(userRef, {
      uid: u.uid,
      email: u.email || null,
      createdAt: serverTimestamp()
    });
  }

  await setDoc(userRef, { lastSeen: serverTimestamp() }, { merge: true });

  return true;
}

/**
 * Create a new user with email and password
 * @param {string} email
 * @param {string} password
 * @returns {Promise}
 */
export function signUp(email, password) {
  return createUserWithEmailAndPassword(auth, email, password);
}

/**
 * Sign in user with email and password
 * @param {string} email
 * @param {string} password
 */
export function signIn(email, password) {
  return signInWithEmailAndPassword(auth, email, password);
}

/**
 * Sign out the current user
 */
export function signOutUser() {
  return signOut(auth);
}

/**
 * Listen for auth state changes
 * @param {(user) => void} cb
 */
export async function onAuthState(cb) {
  return onAuthStateChanged(auth, cb);
}

/**
 * Sign in with Google (popup)
 */
export function signInWithGoogle() {
  const provider = new GoogleAuthProvider();
  return signInWithPopup(auth, provider);
}

/**
 * Sign in with GitHub (popup)
 */
export function signInWithGithub() {
  const provider = new GithubAuthProvider();
  return signInWithPopup(auth, provider);
}

export { auth };