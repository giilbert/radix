import { QueryClient } from "@tanstack/react-query";
import { AxiosError } from "axios";
import { axios } from "@/utils/axios";
import { BACKEND_URL } from "@/utils/consts";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      queryFn: async (ctx) => {
        try {
          const res = await axios.get(BACKEND_URL + "/" + ctx.queryKey);
          return res.data;
        } catch (e: any) {
          const error = e as AxiosError;
          delete error.stack;
          delete (error as any).name;
          throw error;
          // throw `GET [${ctx.queryKey}]: ${error.code}\n${error.response?.data}`;
        }
      },
    },
    mutations: {
      retry: false,
    },
  },
});
