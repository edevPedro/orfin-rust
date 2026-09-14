import { useFocusEffect } from 'expo-router';
import { useCallback, useMemo, useState } from 'react';
import {
  ActivityIndicator,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from 'react-native';
import Animated, { FadeInRight } from 'react-native-reanimated';
import { useSafeAreaInsets } from 'react-native-safe-area-context';

import {
  BrandHeader,
  EmptyState,
  PrimaryButton,
  ScreenBackground,
  SectionTitle,
} from '@/src/components';
import { api } from '@/src/api';
import { getUserId } from '@/src/storage';
import {
  categoryLabels,
  colors,
  formatBRL,
  radius,
  spacing,
  type,
} from '@/src/theme';
import type { ReportSummary } from '@/src/types';

function monthRange(): { from: string; to: string; label: string } {
  const now = new Date();
  const from = new Date(now.getFullYear(), now.getMonth(), 1);
  const to = new Date(now.getFullYear(), now.getMonth() + 1, 0, 23, 59, 59);
  const label = now.toLocaleDateString('pt-BR', { month: 'long', year: 'numeric' });
  return { from: from.toISOString(), to: to.toISOString(), label };
}

export default function ReportsScreen() {
  const insets = useSafeAreaInsets();
  const range = useMemo(() => monthRange(), []);
  const [summary, setSummary] = useState<ReportSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [reconcileMsg, setReconcileMsg] = useState<string | null>(null);
  const [reconciling, setReconciling] = useState(false);

  const load = useCallback(async () => {
    setError(null);
    try {
      const userId = await getUserId();
      const data = await api.reportSummary(userId, range.from, range.to);
      setSummary(data);
    } catch (err) {
      setSummary(null);
      setError(err instanceof Error ? err.message : 'Falha ao carregar relatório');
    } finally {
      setLoading(false);
    }
  }, [range.from, range.to]);

  useFocusEffect(
    useCallback(() => {
      setLoading(true);
      void load();
    }, [load]),
  );

  async function runReconcile() {
    setReconciling(true);
    setReconcileMsg(null);
    try {
      const userId = await getUserId();
      const result = await api.reconcileRun(userId);
      setReconcileMsg(
        result.message ??
          `Comparação concluída: ${result.matched} batendo, ${result.unmatched} sem match.`,
      );
    } catch (err) {
      setReconcileMsg(err instanceof Error ? err.message : 'Falha na comparação');
    } finally {
      setReconciling(false);
    }
  }

  const maxCategory = Math.max(
    1,
    ...(summary?.by_category.map((r) => Number(r.total) || 0) ?? [1]),
  );

  return (
    <ScreenBackground>
      <ScrollView
        contentContainerStyle={[
          styles.container,
          { paddingTop: insets.top + spacing.sm, paddingBottom: insets.bottom + spacing.xl },
        ]}
        refreshControl={
          <RefreshControl
            refreshing={loading}
            onRefresh={() => void load()}
            tintColor={colors.brand}
          />
        }>
        <BrandHeader compact />
        <Text style={styles.month}>{range.label}</Text>

        {loading && !summary ? (
          <ActivityIndicator color={colors.brand} style={{ marginTop: spacing.xxl }} />
        ) : error || !summary ? (
          <EmptyState
            title="Relatório indisponível"
            body={error ?? 'Não há dados para este período ainda.'}
          />
        ) : (
          <>
            <View style={styles.totalBlock}>
              <Text style={styles.totalLabel}>Total gasto</Text>
              <Text style={styles.totalValue}>{formatBRL(summary.total_spent)}</Text>
            </View>

            <SectionTitle title="Por categoria" />
            {summary.by_category.length === 0 ? (
              <Text style={styles.muted}>Sem categorias neste mês.</Text>
            ) : (
              summary.by_category.map((row, i) => {
                const value = Number(row.total) || 0;
                const widthPct = Math.max(8, (value / maxCategory) * 100);
                return (
                  <Animated.View
                    key={row.category}
                    entering={FadeInRight.delay(i * 50)}
                    style={styles.barRow}>
                    <View style={styles.barMeta}>
                      <Text style={styles.barLabel}>
                        {categoryLabels[row.category] ?? row.category}
                      </Text>
                      <Text style={styles.barValue}>{formatBRL(row.total)}</Text>
                    </View>
                    <View style={styles.barTrack}>
                      <View style={[styles.barFill, { width: `${widthPct}%` }]} />
                    </View>
                  </Animated.View>
                );
              })
            )}

            <SectionTitle title="Por origem" />
            {summary.by_source.map((row) => (
              <View key={row.source} style={styles.sourceRow}>
                <Text style={styles.sourceName}>{row.source}</Text>
                <Text style={styles.sourceValue}>
                  {formatBRL(row.total)} · {row.count}
                </Text>
              </View>
            ))}
          </>
        )}

        <View style={styles.reconcile}>
          <SectionTitle title="Open Finance" />
          <Text style={styles.muted}>
            Compara notificações e recibos com o extrato Pluggy no backend.
          </Text>
          <PrimaryButton
            label="Comparar com extrato Open Finance"
            variant="secondary"
            loading={reconciling}
            onPress={() => void runReconcile()}
          />
          {reconcileMsg ? <Text style={styles.reconcileMsg}>{reconcileMsg}</Text> : null}
        </View>
      </ScrollView>
    </ScreenBackground>
  );
}

const styles = StyleSheet.create({
  container: { paddingHorizontal: spacing.lg },
  month: {
    ...type.bodyMedium,
    color: colors.muted,
    textTransform: 'capitalize',
    marginBottom: spacing.lg,
  },
  totalBlock: {
    marginBottom: spacing.xl,
  },
  totalLabel: {
    ...type.label,
    color: colors.muted,
    textTransform: 'uppercase',
  },
  totalValue: {
    ...type.heroAmount,
    color: colors.brand,
    marginTop: spacing.xs,
  },
  muted: {
    ...type.body,
    color: colors.muted,
    marginBottom: spacing.md,
  },
  barRow: { marginBottom: spacing.md },
  barMeta: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 6,
  },
  barLabel: { ...type.bodyMedium, color: colors.ink },
  barValue: { ...type.label, color: colors.muted },
  barTrack: {
    height: 10,
    borderRadius: radius.pill,
    backgroundColor: colors.border,
    overflow: 'hidden',
  },
  barFill: {
    height: '100%',
    borderRadius: radius.pill,
    backgroundColor: colors.brandSoft,
  },
  sourceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    paddingVertical: spacing.sm,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: colors.border,
  },
  sourceName: { ...type.body, color: colors.ink },
  sourceValue: { ...type.caption, color: colors.muted },
  reconcile: { marginTop: spacing.xxl, gap: spacing.sm },
  reconcileMsg: {
    ...type.caption,
    color: colors.success,
    marginTop: spacing.sm,
  },
});
