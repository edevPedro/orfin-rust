import AsyncStorage from '@react-native-async-storage/async-storage';

import { STORAGE_API_URL, STORAGE_USER_ID } from './config';

export async function getUserId(): Promise<string> {
  const existing = await AsyncStorage.getItem(STORAGE_USER_ID);
  if (existing) return existing;
  const created = `user-${Math.random().toString(36).slice(2, 10)}`;
  await AsyncStorage.setItem(STORAGE_USER_ID, created);
  return created;
}

export async function setUserId(userId: string): Promise<void> {
  await AsyncStorage.setItem(STORAGE_USER_ID, userId.trim());
}

export async function getStoredApiUrl(): Promise<string | null> {
  return AsyncStorage.getItem(STORAGE_API_URL);
}

export async function setStoredApiUrl(url: string): Promise<void> {
  await AsyncStorage.setItem(STORAGE_API_URL, url.trim().replace(/\/$/, ''));
}
