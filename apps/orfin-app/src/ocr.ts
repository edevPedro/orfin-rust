import { Platform } from 'react-native';

export type OcrResult = {
  text: string;
  supported: boolean;
};

/** Platform OCR wrapper: ML Kit on native when available; web returns unsupported. */
export async function recognizeImageText(imageUri: string): Promise<OcrResult> {
  if (Platform.OS === 'web') {
    return { text: '', supported: false };
  }

  try {
    const mlkit = await import('expo-mlkit-ocr');
    if (!mlkit.isSupported()) {
      return { text: '', supported: false };
    }
    const result = await mlkit.recognizeText(imageUri);
    return { text: result.text ?? '', supported: true };
  } catch {
    return { text: '', supported: false };
  }
}

export function isNativeOcrAvailable(): boolean {
  if (Platform.OS === 'web') return false;
  try {
    // Sync check only when module is already linked in a dev build.
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const mlkit = require('expo-mlkit-ocr') as { isSupported?: () => boolean };
    return typeof mlkit.isSupported === 'function' ? mlkit.isSupported() : false;
  } catch {
    return false;
  }
}
