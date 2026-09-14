import { NativeModule, requireNativeModule } from 'expo';

import type { BankNotificationsModuleEvents } from './BankNotifications.types';

declare class BankNotificationsModuleType extends NativeModule<BankNotificationsModuleEvents> {
  isEnabled(): Promise<boolean>;
  openSettings(): Promise<void>;
  configure(apiUrl: string, userId: string): Promise<void>;
}

export default requireNativeModule<BankNotificationsModuleType>('BankNotifications');
