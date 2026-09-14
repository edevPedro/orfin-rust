import { Pressable, StyleSheet, Text } from 'react-native';

import { colors, radius, spacing, type } from '@/src/theme';

type Props = {
  label: string;
  selected?: boolean;
  onPress?: () => void;
};

export function Chip({ label, selected, onPress }: Props) {
  return (
    <Pressable
      onPress={onPress}
      style={({ pressed }) => [
        styles.chip,
        selected && styles.selected,
        pressed && styles.pressed,
      ]}>
      <Text style={[styles.label, selected && styles.labelSelected]}>{label}</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  chip: {
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radius.pill,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    backgroundColor: colors.surfaceRaised,
  },
  selected: {
    backgroundColor: colors.brand,
    borderColor: colors.brand,
  },
  pressed: { opacity: 0.85 },
  label: {
    ...type.label,
    color: colors.ink,
  },
  labelSelected: {
    color: '#fff',
  },
});
