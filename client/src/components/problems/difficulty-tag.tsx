import { Tag } from "@chakra-ui/react";

export const DifficultyTag: React.FC<{ difficulty: number }> = ({
  difficulty,
}) => {
  if (difficulty == 0) return <Tag>{difficulty} / 10 Unrated</Tag>;
  if (difficulty < 4)
    return <Tag colorScheme="green">{difficulty} / 10 Easy</Tag>;
  if (difficulty < 8)
    return <Tag colorScheme="yellow">{difficulty} / 10 Medium</Tag>;
  return <Tag colorScheme="red">{difficulty} / 10 Hard</Tag>;
};
