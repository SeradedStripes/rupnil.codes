import { app } from './init.js';
import { getFirestore, doc, setDoc, getDoc, serverTimestamp } from "https://www.gstatic.com/firebasejs/12.8.0/firebase-firestore.js";

const db = getFirestore(app);

export async function requestAlias(alias) {
  const user = (await import('./auth.js')).auth.currentUser;
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
  const u = user || (await import('./auth.js')).auth.currentUser;
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
