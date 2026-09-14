type BankNotificationEvent = {
  packageName: string;
  title: string;
  text: string;
  postedAt: number;
};

async function isEnabled(): Promise<boolean> {
  return false;
}

async function openSettings(): Promise<void> {}

async function configure(_apiUrl: string, _userId: string): Promise<void> {}

function addListener(
  _eventName: 'onBankNotification',
  _listener: (event: BankNotificationEvent) => void,
) {
  return { remove() {} };
}

export default {
  isEnabled,
  openSettings,
  configure,
  addListener,
};
