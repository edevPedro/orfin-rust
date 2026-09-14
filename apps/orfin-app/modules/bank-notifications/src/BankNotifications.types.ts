export type BankNotificationEvent = {
  packageName: string;
  title: string;
  text: string;
  postedAt: number;
};

export type BankNotificationsModuleEvents = {
  onBankNotification: (event: BankNotificationEvent) => void;
};
