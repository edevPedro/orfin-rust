import * as ImagePicker from 'expo-image-picker';
import { useRouter } from 'expo-router';
import { useState } from 'react';
import {
  Platform,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from 'react-native';

import { PrimaryButton, ScreenBackground, SectionTitle } from '@/src/components';
import { api } from '@/src/api';
import { isNativeOcrAvailable, recognizeImageText } from '@/src/ocr';
import { parseExpenseText } from '@/src/parseExpenseText';
import { getUserId } from '@/src/storage';
import { colors, formatBRL, radius, spacing, type } from '@/src/theme';

export default function CaptureScreen() {
  const router = useRouter();
  const [rawText, setRawText] = useState('');
  const [amount, setAmount] = useState('');
  const [merchant, setMerchant] = useState('');
  const [status, setStatus] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const ocrNative = isNativeOcrAvailable();

  function applyParse(text: string) {
    const parsed = parseExpenseText(text);
    if (parsed.amount) setAmount(parsed.amount);
    if (parsed.merchant) setMerchant(parsed.merchant);
    setStatus(
      parsed.amount
        ? `Detectado: ${formatBRL(parsed.amount)}${parsed.merchant ? ` · ${parsed.merchant}` : ''}`
        : 'Texto carregado — ajuste valor e estabelecimento se precisar',
    );
  }

  async function pickAndOcr() {
    setBusy(true);
    setStatus(null);
    try {
      const permission = await ImagePicker.requestMediaLibraryPermissionsAsync();
      if (!permission.granted) {
        setStatus('Permissão da galeria negada');
        return;
      }
      const result = await ImagePicker.launchImageLibraryAsync({
        mediaTypes: ['images'],
        quality: 0.9,
      });
      if (result.canceled || !result.assets?.[0]) return;

      const uri = result.assets[0].uri;
      const ocr = await recognizeImageText(uri);
      if (!ocr.supported || !ocr.text.trim()) {
        setStatus(
          Platform.OS === 'web'
            ? 'OCR nativo indisponível no web — cole o texto do recibo abaixo'
            : 'Não foi possível ler o texto. Cole manualmente ou use um dev build com ML Kit.',
        );
        return;
      }
      setRawText(ocr.text);
      applyParse(ocr.text);
    } catch (err) {
      setStatus(err instanceof Error ? err.message : 'Falha no OCR');
    } finally {
      setBusy(false);
    }
  }

  function onPasteBlur() {
    if (rawText.trim()) applyParse(rawText);
  }

  async function submit() {
    setBusy(true);
    setStatus(null);
    try {
      const userId = await getUserId();
      const parsed = parseExpenseText(rawText);
      const payload = {
        user_id: userId,
        raw_text: rawText.trim() || `${merchant} R$ ${amount}`,
        amount: amount.trim() || parsed.amount || undefined,
        merchant: merchant.trim() || parsed.merchant || undefined,
        currency: 'BRL',
      };
      if (!payload.amount) {
        setStatus('Informe o valor (R$)');
        return;
      }
      const res = await api.fromOcr(payload);
      setStatus('Despesa enviada');
      router.replace(`/payment/${res.payment.id}`);
    } catch (err) {
      setStatus(err instanceof Error ? err.message : 'Falha ao enviar');
    } finally {
      setBusy(false);
    }
  }

  return (
    <ScreenBackground>
      <ScrollView contentContainerStyle={styles.container} keyboardShouldPersistTaps="handled">
        <Text style={styles.lead}>
          Tire foto ou cole o texto do recibo. No aparelho, o ML Kit lê o texto no dispositivo; no
          web, cole o texto manualmente.
        </Text>

        <PrimaryButton
          label={ocrNative ? 'Escolher imagem (OCR)' : 'Escolher imagem'}
          variant="secondary"
          loading={busy}
          onPress={() => void pickAndOcr()}
        />

        <SectionTitle title="Texto do recibo" />
        <TextInput
          style={[styles.input, styles.multiline]}
          value={rawText}
          onChangeText={setRawText}
          onBlur={onPasteBlur}
          placeholder={
            Platform.OS === 'web'
              ? 'Cole aqui: Compra aprovada\nMercado Extra\nR$ 45,90'
              : 'Texto reconhecido ou colado'
          }
          placeholderTextColor={colors.muted}
          multiline
          autoCapitalize="sentences"
        />
        <PrimaryButton
          label="Extrair valor e loja"
          variant="ghost"
          onPress={() => {
            if (rawText.trim()) applyParse(rawText);
            else setStatus('Cole ou reconheça o texto do recibo primeiro');
          }}
        />

        <Text style={styles.label}>Valor (R$)</Text>
        <TextInput
          style={styles.input}
          value={amount}
          onChangeText={setAmount}
          keyboardType="decimal-pad"
          placeholder="0.00"
          placeholderTextColor={colors.muted}
        />

        <Text style={styles.label}>Estabelecimento</Text>
        <TextInput
          style={styles.input}
          value={merchant}
          onChangeText={setMerchant}
          placeholder="Nome da loja"
          placeholderTextColor={colors.muted}
        />

        {status ? <Text style={styles.status}>{status}</Text> : null}

        <PrimaryButton label="Enviar despesa" loading={busy} onPress={() => void submit()} />
      </ScrollView>
    </ScreenBackground>
  );
}

const styles = StyleSheet.create({
  container: { padding: spacing.lg, gap: spacing.sm, paddingBottom: spacing.xxxl },
  lead: { ...type.body, color: colors.muted, marginBottom: spacing.md },
  label: { ...type.label, color: colors.muted, marginTop: spacing.sm },
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
  multiline: {
    minHeight: 120,
    textAlignVertical: 'top',
  },
  status: { ...type.caption, color: colors.brand, marginVertical: spacing.sm },
});
