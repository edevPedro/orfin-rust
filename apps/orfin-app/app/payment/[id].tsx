import { useLocalSearchParams, useRouter } from 'expo-router';
import { useEffect, useState } from 'react';
import {
  ActivityIndicator,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from 'react-native';

import { AmountHero, Chip, EmptyState, PrimaryButton, ScreenBackground } from '@/src/components';
import { api } from '@/src/api';
import { getUserId } from '@/src/storage';
import { colors, formatDateTime, radius, spacing, type } from '@/src/theme';
import type { Category, PaymentEvent } from '@/src/types';

const FALLBACK_CATEGORIES: Category[] = [
  { id: 'alimentacao', label: 'Alimentação' },
  { id: 'transporte', label: 'Transporte' },
  { id: 'moradia', label: 'Moradia' },
  { id: 'lazer', label: 'Lazer' },
  { id: 'saude', label: 'Saúde' },
  { id: 'educacao', label: 'Educação' },
  { id: 'assinaturas', label: 'Assinaturas' },
  { id: 'outros', label: 'Outros' },
];

export default function PaymentExplainScreen() {
  const { id } = useLocalSearchParams<{ id: string }>();
  const router = useRouter();
  const [payment, setPayment] = useState<PaymentEvent | null>(null);
  const [categories, setCategories] = useState<Category[]>([]);
  const [category, setCategory] = useState('outros');
  const [note, setNote] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      try {
        const userId = await getUserId();
        const [awaiting, all, cats] = await Promise.all([
          api.listAwaiting(userId),
          api.listPayments(userId),
          api.categories(),
        ]);
        const found =
          awaiting.payments.find((p) => p.id === id) ??
          all.payments.find((p) => p.id === id) ??
          null;
        setPayment(found);
        setCategories(cats.categories ?? []);
        setCategory(found?.suggested_category ?? found?.category ?? 'outros');
        setNote(found?.user_note ?? '');
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Falha ao carregar');
      } finally {
        setLoading(false);
      }
    })();
  }, [id]);

  async function submit() {
    if (!id) return;
    setSaving(true);
    setError(null);
    try {
      await api.explain(id, category, note.trim() || undefined);
      router.back();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Falha ao salvar');
    } finally {
      setSaving(false);
    }
  }

  if (loading) {
    return (
      <ScreenBackground>
        <View style={styles.center}>
          <ActivityIndicator color={colors.brand} />
        </View>
      </ScreenBackground>
    );
  }

  if (!payment) {
    return (
      <ScreenBackground>
        <EmptyState title="Pagamento não encontrado" body={error ?? undefined} />
      </ScreenBackground>
    );
  }

  const chips = categories.length > 0 ? categories : FALLBACK_CATEGORIES;

  return (
    <ScreenBackground>
      <ScrollView contentContainerStyle={styles.container} keyboardShouldPersistTaps="handled">
        <AmountHero
          amount={payment.amount}
          merchant={payment.merchant ?? payment.description}
          subtitle={formatDateTime(payment.paid_at)}
        />

        <Text style={styles.label}>Categoria</Text>
        <View style={styles.chips}>
          {chips.map((item) => (
            <Chip
              key={item.id}
              label={item.label}
              selected={category === item.id}
              onPress={() => setCategory(item.id)}
            />
          ))}
        </View>

        <Text style={styles.label}>O que foi?</Text>
        <TextInput
          style={styles.input}
          value={note}
          onChangeText={setNote}
          placeholder="Ex: almoço com o time"
          placeholderTextColor={colors.muted}
          multiline
        />

        {error && <Text style={styles.error}>{error}</Text>}

        <PrimaryButton
          label={saving ? 'Salvando…' : 'Organizar'}
          loading={saving}
          onPress={() => void submit()}
        />
      </ScrollView>
    </ScreenBackground>
  );
}

const styles = StyleSheet.create({
  center: { flex: 1, alignItems: 'center', justifyContent: 'center' },
  container: { padding: spacing.lg, paddingBottom: spacing.xxxl },
  label: {
    ...type.section,
    color: colors.ink,
    textTransform: 'uppercase',
    marginTop: spacing.lg,
    marginBottom: spacing.md,
  },
  chips: { flexDirection: 'row', flexWrap: 'wrap', gap: spacing.sm },
  input: {
    minHeight: 96,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radius.md,
    padding: spacing.md,
    textAlignVertical: 'top',
    backgroundColor: colors.surfaceRaised,
    ...type.body,
    color: colors.ink,
    marginBottom: spacing.lg,
  },
  error: { ...type.caption, color: colors.danger, marginBottom: spacing.md },
});
