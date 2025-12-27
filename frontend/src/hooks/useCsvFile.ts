import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { csvApi } from '@/api/csvApi';
import { useCsvStore } from '@/stores/csvStore';

/**
 * 使用CSV文件的Hook
 */
export function useCsvFile(filePath: string | null) {
  const queryClient = useQueryClient();
  const { setFileInfo, setCurrentFile } = useCsvStore();

  // 查询文件信息
  const fileInfoQuery = useQuery({
    queryKey: ['csv-file-info', filePath],
    queryFn: () => csvApi.getFileInfo(filePath!),
    enabled: !!filePath,
    staleTime: Infinity, // 文件信息不会改变
  });

  // 打开文件Mutation
  const openFileMutation = useMutation({
    mutationFn: (path: string) => csvApi.openFile(path),
    onSuccess: (data) => {
      setCurrentFile(data.file_path);
      setFileInfo(data);
      queryClient.setQueryData(['csv-file-info', data.file_path], data);
    },
  });

  // 关闭文件Mutation
  const closeFileMutation = useMutation({
    mutationFn: (path: string) => csvApi.closeFile(path),
    onSuccess: () => {
      setCurrentFile(null);
      setFileInfo(null);
      queryClient.removeQueries({ queryKey: ['csv-file', filePath] });
    },
  });

  return {
    fileInfo: fileInfoQuery.data,
    isLoading: fileInfoQuery.isLoading,
    error: fileInfoQuery.error,
    openFile: openFileMutation.mutateAsync,
    closeFile: closeFileMutation.mutateAsync,
    isOpening: openFileMutation.isPending,
  };
}

/**
 * 使用CSV页面的Hook
 */
export function useCsvPage(filePath: string | null, page: number, pageSize: number) {
  return useQuery({
    queryKey: ['csv-page', filePath, page, pageSize],
    queryFn: () => csvApi.readPage(filePath!, page, pageSize),
    enabled: !!filePath,
    staleTime: 5 * 60 * 1000, // 5分钟缓存
    placeholderData: (previousData) => previousData, // 保持上一页数据，避免闪烁（v5语法）
  });
}

