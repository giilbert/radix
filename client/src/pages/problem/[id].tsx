import { Layout } from "@/components/layout/layout";
import { ProblemEditor } from "@/components/problems/problem-editor";
import { ProblemView } from "@/components/problems/problem-view";
import { Problem, TestCase } from "@/types/problem";
import { Box, Button, HStack } from "@chakra-ui/react";
import { useQuery } from "@tanstack/react-query";
import { NextPage } from "next";
import { useSession } from "next-auth/react";
import { useRouter } from "next/router";
import { useState } from "react";
import { FiArrowLeft, FiEdit } from "react-icons/fi";

const ProblemPage: NextPage = () => {
  const router = useRouter();
  const { data: session } = useSession();
  const [isEditing, setIsEditing] = useState(false);
  const problemQuery = useQuery<
    Problem & {
      testCases: TestCase[];
    }
  >([`problem/${router.query.id as string}`], {
    enabled: !!router.query.id,
  });

  return (
    <Layout title="Problem" selectedPage="problems">
      <Button leftIcon={<FiArrowLeft />} onClick={router.back} mr="2">
        Back
      </Button>
      {problemQuery.isSuccess &&
        problemQuery.data.author.id === session?.user.id && (
          <>
            {!isEditing && (
              <Button leftIcon={<FiEdit />} onClick={() => setIsEditing(true)}>
                Edit
              </Button>
            )}
          </>
        )}

      {problemQuery.status === "success" && (
        <>
          {isEditing ? (
            <ProblemEditor
              problem={problemQuery.data}
              setIsEditing={setIsEditing}
            />
          ) : (
            <Box mt="2">
              <ProblemView problem={problemQuery.data} />
            </Box>
          )}
        </>
      )}
    </Layout>
  );
};

export default ProblemPage;
