import { useFocusEffect } from 'expo-router';
import { useCallback, useState } from 'react';
import { ActivityIndicator, FlatList, RefreshControl, StyleSheet, View } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';

import { BrandHeader, EmptyState, PaymentRow, ScreenBackground, SectionTitle } from '@/src/components';
import { api } from '@/src/api';
import { getUserId } from '@/src/storage';
import { colors, spacing } from '@/src/theme';
import type { PaymentEvent } from '@/src/types';

export default function PaymentsScreen() {
  const insets = useSafeAreaInsets();
  const [payments, setPayments] = useState<PaymentEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      const userId = await getUserId();
      const data = await api.listPayments(userId);
      setPayments(data.payments ?? []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Falha ao carregar');
    } finally {
      setLoading(false);
    }
  }, []);

  useFocusEffect(
    useCallback(() => {
      setLoading(true);
      void load();
    }, [load]),
  );

  if (loading && payments.length === 0) {
    return (
      <ScreenBackground>
        <View style={[styles.center, { paddingTop: insets.top }]}>
          <ActivityIndicator color={colors.brand} />
        </View>
      </ScreenBackground>
    );
  }

  return (
    <ScreenBackground>
      <FlatList
        contentContainerStyle={[
          styles.list,
          { paddingTop: insets.top + spacing.sm, paddingBottom: insets.bottom + spacing.xl },
          payments.length === 0 && styles.flexGrow,
        ]}
        data={payments}
        keyExtractor={(item) => item.id}
        refreshControl={
          <RefreshControl
            refreshing={loading}
            onRefresh={() => void load()}
            tintColor={colors.brand}
          />
        }
        ListHeaderComponent={
          <View>
            <BrandHeader compact />
            <SectionTitle title="Histórico" action={`${payments.length} itens`} />
          </View>
        }
        ListEmptyComponent={
          <EmptyState
            title={error ? 'Falha ao carregar' : 'Sem pagamentos ainda'}
            body={error ?? 'Seus pagamentos organizados e pendentes aparecem aqui.'}
          />
        }
        renderItem={({ item, index }) => (
          <PaymentRow payment={item} index={index} showStatus />
        )}
      />
    </ScreenBackground>
  );
}

const styles = StyleSheet.create({
  center: { flex: 1, alignItems: 'center', justifyContent: 'center' },
  list: { paddingHorizontal: spacing.lg },
  flexGrow: { flexGrow: 1 },
});
