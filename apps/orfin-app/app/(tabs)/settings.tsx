import { useFocusEffect } from 'expo-router';
import { useCallback, useState } from 'react';
import {
  Alert,
  Platform,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';

import {
  BrandHeader,
  PrimaryButton,
  ScreenBackground,
  SectionTitle,
} from '@/src/components';
import { api } from '@/src/api';
import {
  configureListener,
  isListenerEnabled,
  isListenerSupported,
  openListenerSettings,
} from '@/src/bankListener';
import { API_URL } from '@/src/config';
import { getStoredApiUrl, getUserId, setStoredApiUrl, setUserId } from '@/src/storage';
import { colors, radius, spacing, type } from '@/src/theme';
import type { AlertCheckResult, AlertRule } from '@/src/types';

export default function SettingsScreen() {
  const insets = useSafeAreaInsets();
  const [userId, setUserIdState] = useState('');
  const [apiUrl, setApiUrl] = useState(API_URL);
  const [listenerOn, setListenerOn] = useState(false);
  const [telegram, setTelegram] = useState('');
  const [discord, setDiscord] = useState('');
  const [threshold, setThreshold] = useState('500');
  const [alertCategory, setAlertCategory] = useState('');
  const [alerts, setAlerts] = useState<AlertRule[]>([]);
  const [checkResult, setCheckResult] = useState<AlertCheckResult | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    const id = await getUserId();
    setUserIdState(id);
    const storedUrl = await getStoredApiUrl();
    if (storedUrl) setApiUrl(storedUrl);
    if (isListenerSupported()) {
      setListenerOn(await isListenerEnabled());
      await configureListener(storedUrl ?? API_URL, id);
    }
    try {
      const listed = await api.listAlerts(id);
      setAlerts(listed.alerts ?? []);
    } catch {
      // alerts optional offline
    }
  }, []);

  useFocusEffect(
    useCallback(() => {
      void refresh();
    }, [refresh]),
  );

  async function saveIdentity() {
    await setUserId(userId);
    await setStoredApiUrl(apiUrl);
    await configureListener(apiUrl, userId);
    setStatus('Salvo — o serviço nativo posta sozinho em background');
  }

  async function saveChannel(channel: 'telegram' | 'discord', target: string) {
    try {
      const id = await getUserId();
      await api.linkChannel(id, channel, target.trim());
      setStatus(`${channel} vinculado`);
    } catch (err) {
      Alert.alert('Erro', err instanceof Error ? err.message : 'Falha');
    }
  }

  async function createBudgetAlert() {
    setBusy(true);
    try {
      const id = await getUserId();
      await api.createAlert({
        user_id: id,
        kind: 'budget_threshold',
        threshold: threshold.replace(',', '.'),
        category: alertCategory.trim() || null,
      });
      const listed = await api.listAlerts(id);
      setAlerts(listed.alerts ?? []);
      setStatus('Regra de alerta criada');
    } catch (err) {
      Alert.alert('Erro', err instanceof Error ? err.message : 'Falha ao criar alerta');
    } finally {
      setBusy(false);
    }
  }

  async function runAlertCheck() {
    setBusy(true);
    try {
      const id = await getUserId();
      const result = await api.checkAlerts(id);
      setCheckResult(result);
    } catch (err) {
      Alert.alert('Erro', err instanceof Error ? err.message : 'Falha ao verificar alertas');
    } finally {
      setBusy(false);
    }
  }

  return (
    <ScreenBackground>
      <ScrollView
        contentContainerStyle={[
          styles.container,
          { paddingTop: insets.top + spacing.sm, paddingBottom: insets.bottom + spacing.xxl },
        ]}
        keyboardShouldPersistTaps="handled">
        <BrandHeader compact />

        <SectionTitle title="Conta" />
        <Text style={styles.label}>user_id</Text>
        <TextInput
          style={styles.input}
          value={userId}
          onChangeText={setUserIdState}
          autoCapitalize="none"
          placeholderTextColor={colors.muted}
        />
        <Text style={styles.label}>API URL</Text>
        <TextInput
          style={styles.input}
          value={apiUrl}
          onChangeText={setApiUrl}
          autoCapitalize="none"
          placeholderTextColor={colors.muted}
        />
        <PrimaryButton label="Salvar" onPress={() => void saveIdentity()} />

        <SectionTitle title="Alertas de orçamento" />
        <Text style={styles.help}>
          Avisa quando o gasto do mês ultrapassar o limite (opcionalmente por categoria).
        </Text>
        <Text style={styles.label}>Limite (R$)</Text>
        <TextInput
          style={styles.input}
          value={threshold}
          onChangeText={setThreshold}
          keyboardType="decimal-pad"
          placeholderTextColor={colors.muted}
        />
        <Text style={styles.label}>Categoria (opcional)</Text>
        <TextInput
          style={styles.input}
          value={alertCategory}
          onChangeText={setAlertCategory}
          autoCapitalize="none"
          placeholder="ex: alimentacao"
          placeholderTextColor={colors.muted}
        />
        <PrimaryButton
          label="Criar regra"
          loading={busy}
          onPress={() => void createBudgetAlert()}
        />
        <PrimaryButton
          label="Verificar alertas agora"
          variant="secondary"
          loading={busy}
          onPress={() => void runAlertCheck()}
        />

        {alerts.length > 0 && (
          <View style={styles.block}>
            {alerts.map((a) => (
              <Text key={a.id} style={styles.alertLine}>
                Limite {a.threshold}
                {a.category ? ` · ${a.category}` : ' · geral'} ·{' '}
                {a.enabled ? 'ativo' : 'off'}
              </Text>
            ))}
          </View>
        )}

        {checkResult && (
          <View style={styles.checkBox}>
            <Text style={styles.checkTitle}>
              {checkResult.triggered ? 'Alerta disparado' : 'Dentro do orçamento'}
            </Text>
            {checkResult.messages.map((m, i) => (
              <Text key={i} style={styles.help}>
                {m}
              </Text>
            ))}
          </View>
        )}

        <SectionTitle title="NotificationListener" />
        <Text style={styles.help}>
          {isListenerSupported()
            ? listenerOn
              ? 'Acesso a notificações ativo. O serviço nativo envia compras ao backend (sem JS).'
              : 'Ative o acesso às notificações do Orfin nas configurações do sistema.'
            : 'No iOS/web use Pluggy (Open Finance). NotificationListener é só Android.'}
        </Text>
        {isListenerSupported() && (
          <PrimaryButton
            label="Abrir configurações"
            variant="secondary"
            onPress={() => void openListenerSettings()}
          />
        )}

        <SectionTitle title="Canais" />
        <Text style={styles.label}>Telegram chat_id</Text>
        <TextInput
          style={styles.input}
          value={telegram}
          onChangeText={setTelegram}
          autoCapitalize="none"
          placeholderTextColor={colors.muted}
        />
        <PrimaryButton
          label="Vincular Telegram"
          variant="secondary"
          onPress={() => void saveChannel('telegram', telegram)}
        />
        <Text style={styles.label}>Discord webhook URL</Text>
        <TextInput
          style={styles.input}
          value={discord}
          onChangeText={setDiscord}
          autoCapitalize="none"
          placeholderTextColor={colors.muted}
        />
        <PrimaryButton
          label="Vincular Discord"
          variant="secondary"
          onPress={() => void saveChannel('discord', discord)}
        />

        {status && <Text style={styles.status}>{status}</Text>}
        <Text style={styles.footer}>Plataforma: {Platform.OS}</Text>
      </ScrollView>
    </ScreenBackground>
  );
}

const styles = StyleSheet.create({
  container: { paddingHorizontal: spacing.lg, gap: spacing.sm },
  label: { ...type.label, color: colors.muted, marginTop: spacing.sm },
  help: { ...type.body, color: colors.muted },
  input: {
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radius.md,
    paddingHorizontal: spacing.md,
    paddingVertical: 12,
    backgroundColor: colors.surfaceRaised,
    ...type.body,
    color: colors.ink,
  },
  block: { marginTop: spacing.sm, gap: 4 },
  alertLine: { ...type.caption, color: colors.ink },
  checkBox: {
    marginTop: spacing.md,
    padding: spacing.lg,
    borderRadius: radius.lg,
    backgroundColor: colors.tealSoft,
    gap: 4,
  },
  checkTitle: { ...type.bodyMedium, color: colors.brand },
  status: { ...type.caption, color: colors.success, marginTop: spacing.md },
  footer: { ...type.caption, color: colors.muted, marginTop: spacing.xl },
});
