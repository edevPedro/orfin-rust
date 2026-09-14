import { Platform } from 'react-native';

import BankNotificationsModule from '../modules/bank-notifications';

export function isListenerSupported(): boolean {
  return Platform.OS === 'android';
}

export async function isListenerEnabled(): Promise<boolean> {
  if (!isListenerSupported()) return false;
  return BankNotificationsModule.isEnabled();
}

export async function openListenerSettings(): Promise<void> {
  if (!isListenerSupported()) return;
  await BankNotificationsModule.openSettings();
}

/** Persiste apiUrl/userId para o NotificationListenerService postar sem JS. */
export async function configureListener(apiUrl: string, userId: string): Promise<void> {
  if (!isListenerSupported()) return;
  await BankNotificationsModule.configure(apiUrl, userId);
}
