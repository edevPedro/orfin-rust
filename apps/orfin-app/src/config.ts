import { Platform } from 'react-native';

const fallback =
  Platform.OS === 'android' ? 'http://10.0.2.2:3000' : 'http://localhost:3000';

export const API_URL = process.env.EXPO_PUBLIC_API_URL ?? fallback;
export const STORAGE_USER_ID = 'orfin.user_id';
export const STORAGE_API_URL = 'orfin.api_url';
