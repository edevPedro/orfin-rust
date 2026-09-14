import { Link, Stack } from 'expo-router';
import { StyleSheet, Text, View } from 'react-native';

import { colors, type } from '@/src/theme';

export default function NotFoundScreen() {
  return (
    <>
      <Stack.Screen options={{ title: 'Não encontrado' }} />
      <View style={styles.container}>
        <Text style={styles.title}>Tela inexistente</Text>
        <Link href="/" style={styles.link}>
          <Text style={styles.linkText}>Voltar ao Orfin</Text>
        </Link>
      </View>
    </>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    padding: 20,
    backgroundColor: colors.mistBottom,
  },
  title: { ...type.title, color: colors.ink },
  link: { marginTop: 15, paddingVertical: 15 },
  linkText: { ...type.bodyMedium, color: colors.brand },
});
