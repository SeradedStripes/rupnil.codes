
// Import the functions you need from the SDKs you need
import { initializeApp } from "https://www.gstatic.com/firebasejs/12.8.0/firebase-app.js";
import { getAnalytics } from "https://www.gstatic.com/firebasejs/12.8.0/firebase-analytics.js";
import {
  getAuth,
  createUserWithEmailAndPassword,
  signInWithEmailAndPassword,
  signOut,
  onAuthStateChanged
} from "https://www.gstatic.com/firebasejs/12.8.0/firebase-auth.js";

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
export function onAuthState(cb) {
  return onAuthStateChanged(auth, cb);
}

export { auth };