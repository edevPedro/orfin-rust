import { Link, useFocusEffect } from 'expo-router';
import { useCallback, useState } from 'react';
import {
  ActivityIndicator,
  FlatList,
  Pressable,
  RefreshControl,
  StyleSheet,
  Text,
  View,
} from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';

import { BrandHeader, EmptyState, PaymentRow, ScreenBackground } from '@/src/components';
import { api } from '@/src/api';
import { getUserId } from '@/src/storage';
import { colors, spacing, type } from '@/src/theme';
import type { PaymentEvent } from '@/src/types';

export default function AwaitingScreen() {
  const insets = useSafeAreaInsets();
  const [payments, setPayments] = useState<PaymentEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setError(null);
    try {
      const userId = await getUserId();
      const data = await api.listAwaiting(userId);
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
            <BrandHeader />
            <View style={styles.captureRow}>
              <Link href="/capture" asChild>
                <Pressable style={({ pressed }) => [styles.captureBtn, pressed && styles.pressed]}>
                  <Text style={styles.captureText}>Capturar recibo</Text>
                </Pressable>
              </Link>
            </View>
          </View>
        }
        ListEmptyComponent={
          <EmptyState
            title={error ? 'Não foi possível carregar' : 'Tudo em dia'}
            body={
              error ??
              'Quando o banco notificar uma compra, ela aparece aqui para você organizar.'
            }
          />
        }
        renderItem={({ item, index }) => <PaymentRow payment={item} index={index} />}
      />
    </ScreenBackground>
  );
}

const styles = StyleSheet.create({
  center: { flex: 1, alignItems: 'center', justifyContent: 'center' },
  list: { paddingHorizontal: spacing.lg },
  flexGrow: { flexGrow: 1 },
  captureRow: { marginBottom: spacing.lg },
  captureBtn: {
    alignSelf: 'flex-start',
    paddingVertical: spacing.sm,
    paddingHorizontal: spacing.md,
    borderRadius: 999,
    backgroundColor: colors.tealSoft,
  },
  pressed: { opacity: 0.85 },
  captureText: {
    ...type.label,
    color: colors.brand,
  },
});
