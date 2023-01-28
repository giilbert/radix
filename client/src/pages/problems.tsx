import { Layout } from "@/components/layout/layout";
import { debounce } from "throttle-debounce";
import { AxiosErrorMessage } from "@/components/ui/axios-error-message";
import { OnBottom } from "@/components/ui/on-bottom";
import { ListingProblem } from "@/types/problem";
import { axios } from "@/utils/axios";
import {
  Box,
  Button,
  Heading,
  HStack,
  SimpleGrid,
  Tag,
  Text,
} from "@chakra-ui/react";
import { useInfiniteQuery, useMutation } from "@tanstack/react-query";
import { AxiosError } from "axios";
import { NextPage } from "next";
import Link from "next/link";
import { FiPlus } from "react-icons/fi";
import { DifficultyTag } from "@/components/problems/difficulty-tag";

const ProblemsPage: NextPage = () => {
  const problemsQuery = useInfiniteQuery<ListingProblem[], AxiosError>(
    ["problem/infinite"],
    async (ctx) => {
      const res = await axios.get(
        `problem/infinite${ctx.pageParam ? "?cursor=" + ctx.pageParam : ""}`
      );

      return res.data;
    },
    {
      getNextPageParam: (current) => current.at(-1)?.id,
      refetchOnMount: false,
      refetchOnReconnect: false,
      refetchOnWindowFocus: false,
    }
  );
  const problems = problemsQuery.data?.pages.flat();

  const createProblem = useMutation<{ id: string }, AxiosError>(
    ["problem"],
    async () => {
      const response = await axios.post("/problem");
      return response.data;
    }
  );

  return (
    <Layout title="Problems" selectedPage="problems">
      <HStack w="full">
        <Heading>Problems</Heading>

        <Button
          ml="auto !important"
          isLoading={createProblem.isLoading}
          onClick={() => {
            createProblem.mutateAsync().then((a) => {
              problemsQuery.refetch();
              console.log(a);
            });
          }}
          leftIcon={<FiPlus size={20} />}
        >
          Create
        </Button>
      </HStack>

      {problemsQuery.status === "error" && (
        <AxiosErrorMessage error={problemsQuery.error} />
      )}

      {problems && (
        <OnBottom
          onBottom={debounce(500, () => {
            problemsQuery.fetchNextPage();
          })}
        >
          <SimpleGrid
            mt="2"
            templateColumns={{
              base: "1fr",
              sm: "repeat(2, 1fr)",
              md: "repeat(3, 1fr)",
            }}
            gap="2"
          >
            {problems.map((problem) => (
              <Link
                key={problem.id}
                href={`/problem/${problem.id}`}
                style={{
                  width: "100%",
                }}
              >
                <Box
                  w="full"
                  h="full"
                  bg="whiteAlpha.100"
                  px="4"
                  py="3"
                  borderRadius="md"
                  gap="2"
                  transition="transform 100ms ease-in-out, background 300ms"
                  _hover={{
                    cursor: "pointer",
                    transform: "scale(101%)",
                    bg: "whiteAlpha.200",
                  }}
                >
                  <HStack mb="2" alignItems="flex-end">
                    <Heading fontSize="xl">{problem.title}</Heading>

                    <Text>by {problem.author.name}</Text>
                  </HStack>

                  <HStack>
                    {problem.draft && <Tag colorScheme="red">DRAFT</Tag>}
                    <DifficultyTag difficulty={problem.difficulty} />
                  </HStack>
                </Box>
              </Link>
            ))}
          </SimpleGrid>

          <Text my="8" textAlign="center" color="whiteAlpha.500" fontSize="xl">
            {problemsQuery.hasNextPage
              ? "Loading more.."
              : "You reached the end"}
          </Text>
        </OnBottom>
      )}
    </Layout>
  );
};

export default ProblemsPage;
